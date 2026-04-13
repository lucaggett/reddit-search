mod arguments;
mod constants;
mod line_processing;

use crate::arguments::CommandLineArgs;
use crate::line_processing::process_chunk;
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{sync_channel, Receiver, SyncSender};
use std::sync::Arc;
use std::thread;
use zstd::Decoder;

// How much decompressed data each worker chunk holds. 4 MiB is large enough
// to amortize channel + memory-allocation overhead and small enough that
// keeping a handful of in-flight chunks per worker stays well under memory
// limits (N workers * 3 in-flight * 4 MiB = ~150 MiB for 12 workers).
const READ_CHUNK_BYTES: usize = 4 << 20;
// Size of each individual read() into the zstd decoder.
const READ_BUF_BYTES: usize = 256 << 10;
// Per-worker channel depth. 2 gives the reader a little runway so workers
// don't starve, but keeps memory pressure bounded.
const WORKER_CHANNEL_DEPTH: usize = 2;

// Wraps the raw compressed File so the progress bar can track exact
// compressed-byte position. The counter is Arc-shared with the main thread.
struct CountingReader<R> {
    inner: R,
    counter: Arc<AtomicU64>,
}

impl<R: Read> Read for CountingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.inner.read(buf)?;
        self.counter.fetch_add(n as u64, Ordering::Relaxed);
        Ok(n)
    }
}

type ZstdReader = Decoder<'static, std::io::BufReader<CountingReader<File>>>;

fn open_zstd_reader(path: &Path) -> io::Result<(ZstdReader, Arc<AtomicU64>)> {
    let file = File::open(path)?;
    let counter = Arc::new(AtomicU64::new(0));
    let counting = CountingReader {
        inner: file,
        counter: counter.clone(),
    };
    let mut decoder = Decoder::new(counting)?;
    decoder.window_log_max(31)?;
    Ok((decoder, counter))
}

fn count_lines(file_name: &str) -> io::Result<()> {
    let path = PathBuf::from(file_name);
    let metadata = path.metadata()?;
    let (mut reader, _) = open_zstd_reader(&path)?;
    let mut buf = vec![0u8; READ_BUF_BYTES];
    let mut n_lines: u64 = 0;
    loop {
        match reader.read(&mut buf)? {
            0 => break,
            n => n_lines += memchr::memchr_iter(b'\n', &buf[..n]).count() as u64,
        }
    }
    println!("{};{};{}", file_name, metadata.len(), n_lines);
    Ok(())
}

fn format_elapsed(secs: u64) -> String {
    if secs < 60 {
        format!("{} seconds", secs)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        let unit = if m == 1 { "minute" } else { "minutes" };
        format!("{} {}, {} seconds", m, unit, s)
    }
}

fn build_search_strings(fields: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::with_capacity(fields.len() * 2);
    for field in fields {
        let Some((key, value)) = field.split_once(':') else {
            return Err(format!(
                "Field {} is not in the format <field>:<value>",
                field
            ));
        };
        let key = key.to_lowercase();
        let value = value.to_lowercase();
        let unquoted = value.parse::<i64>().is_ok()
            || value == "true"
            || value == "false"
            || value == "null";
        if unquoted {
            out.push(format!("\"{}\": {}", key, value));
            out.push(format!("\"{}\":{}", key, value));
        } else {
            out.push(format!("\"{}\": \"{}\"", key, value));
            out.push(format!("\"{}\":\"{}\"", key, value));
        }
    }
    Ok(out)
}

// Messages that flow reader -> worker and worker -> writer. `End` is the
// terminal sentinel that lets the writer follow strict round-robin order
// without needing a separate "total chunks" signal.
enum WorkMsg {
    Chunk(Vec<u8>),
    End,
}

enum ResultMsg {
    Chunk { bytes: Vec<u8>, matches: usize },
    End,
}

fn spawn_worker(
    work_rx: Receiver<WorkMsg>,
    result_tx: SyncSender<ResultMsg>,
    pool_tx: SyncSender<Vec<u8>>,
    ac: Arc<AhoCorasick>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        // Thread-local state persists for the worker's lifetime — scratch
        // holds the ascii-lowercased copy of the current line, `out` the
        // accumulated matched bytes for the current chunk. Both grow to
        // watermark on the first chunk and never allocate again.
        let mut scratch: Vec<u8> = Vec::with_capacity(8 << 10);
        let mut out: Vec<u8> = Vec::with_capacity(64 << 10);

        while let Ok(msg) = work_rx.recv() {
            match msg {
                WorkMsg::Chunk(buf) => {
                    out.clear();
                    let n = process_chunk(&buf, &ac, &mut scratch, &mut out);
                    // Hand off a tightly sized Vec to the writer while
                    // keeping `out`'s capacity for the next chunk.
                    let result_bytes = out.clone();
                    if result_tx
                        .send(ResultMsg::Chunk {
                            bytes: result_bytes,
                            matches: n,
                        })
                        .is_err()
                    {
                        return;
                    }
                    // Return the input buffer to the reader's pool.
                    if pool_tx.send(buf).is_err() {
                        return;
                    }
                }
                WorkMsg::End => {
                    let _ = result_tx.send(ResultMsg::End);
                    return;
                }
            }
        }
    })
}

fn main() -> io::Result<()> {
    let mut args = CommandLineArgs::new().unwrap();

    if args.linecount {
        return count_lines(&args.input);
    }

    let search_fields: Vec<String> = if let Some(ref preset) = args.preset {
        match arguments::get_preset_fields(preset) {
            Some(f) => f,
            None => return Ok(()),
        }
    } else {
        args.fields.as_ref().unwrap().clone()
    };

    let search_strings = match build_search_strings(&search_fields) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return Ok(());
        }
    };

    let ac = AhoCorasickBuilder::new()
        .match_kind(MatchKind::LeftmostFirst)
        .build(&search_strings)
        .expect("Failed to build Aho-Corasick automaton");
    let ac = Arc::new(ac);

    let input_path = PathBuf::from(&args.input);
    if !input_path.exists() {
        eprintln!("Input file {} does not exist.", args.input);
        return Ok(());
    }
    let metadata = input_path.metadata()?;
    if !metadata.is_file() {
        eprintln!("Input file {} is not a regular file.", args.input);
        return Ok(());
    }

    let (reader, bytes_read) = open_zstd_reader(&input_path)?;

    let output_path = PathBuf::from(&args.output);
    if output_path.exists() && !args.append && !args.overwrite {
        eprint!(
            "File {} already exists. Enter 'a' to append to the file, 'o' to overwrite, or anything else to exit: ",
            args.output
        );
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        match user_input.trim() {
            "a" => args.append = true,
            "o" => args.append = false,
            _ => {
                println!("Exiting");
                return Ok(());
            }
        }
    }
    if !args.append && output_path.exists() {
        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&output_path)?;
    }
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(&output_path)?;

    let num_workers = args.threads.max(1);

    if args.verbose {
        println!(
            "Starting reddit-search for {} ({} workers) at {}",
            args.input,
            num_workers,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        println!("Input file: {}", args.input);
        println!("Output file: {}", args.output);
        println!("Append: {}", args.append);
        println!("Workers: {}", num_workers);
        println!("Compressed size: {} bytes", metadata.len());
        println!("Search strings: {}", search_strings.join(", "));
    }

    let pb = ProgressBar::new(metadata.len());
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} | {percent}% | {eta} left",
            )
            .expect("Failed to set progress bar style")
            .progress_chars("=> "),
    );

    let mut output_stream = BufWriter::with_capacity(1 << 20, output_file);

    // Buffer pool for the reader's 4 MiB input chunks. Prime it with
    // 3 * num_workers buffers so the reader has a little head-room and
    // pages don't get faulted-in every chunk.
    let pool_size = num_workers * 3;
    let (pool_tx, pool_rx) = sync_channel::<Vec<u8>>(pool_size);
    for _ in 0..pool_size {
        pool_tx
            .send(Vec::with_capacity(READ_CHUNK_BYTES + 1024))
            .expect("failed to prime buffer pool");
    }

    // Per-worker work/result channels. Round-robin dispatch by the reader
    // and round-robin collection by the writer preserves input file order
    // in the output without any reorder buffer.
    let mut work_txs: Vec<SyncSender<WorkMsg>> = Vec::with_capacity(num_workers);
    let mut result_rxs: Vec<Receiver<ResultMsg>> = Vec::with_capacity(num_workers);
    let mut worker_handles: Vec<thread::JoinHandle<()>> = Vec::with_capacity(num_workers);

    for _ in 0..num_workers {
        let (work_tx, work_rx) = sync_channel::<WorkMsg>(WORKER_CHANNEL_DEPTH);
        let (result_tx, result_rx) = sync_channel::<ResultMsg>(WORKER_CHANNEL_DEPTH);
        let handle = spawn_worker(work_rx, result_tx, pool_tx.clone(), ac.clone());
        work_txs.push(work_tx);
        result_rxs.push(result_rx);
        worker_handles.push(handle);
    }

    // Reader thread: pulls empty buffers from the pool, fills them from
    // the zstd decoder, slices at the last newline so workers never see a
    // split line, and dispatches round-robin to workers. After EOF, sends
    // an End sentinel to every worker.
    let reader_handle = {
        let pool_tx_reader = pool_tx.clone();
        thread::spawn(move || -> io::Result<()> {
            let mut reader = reader;
            let mut carryover: Vec<u8> = Vec::new();
            let mut tmp = vec![0u8; READ_BUF_BYTES];
            let mut worker_idx = 0usize;
            loop {
                let mut buf = match pool_rx.recv() {
                    Ok(v) => v,
                    Err(_) => break, // workers have all exited
                };
                buf.clear();
                if !carryover.is_empty() {
                    buf.append(&mut carryover);
                }
                let mut eof = false;
                while buf.len() < READ_CHUNK_BYTES {
                    match reader.read(&mut tmp) {
                        Ok(0) => {
                            eof = true;
                            break;
                        }
                        Ok(n) => buf.extend_from_slice(&tmp[..n]),
                        Err(e) => return Err(e),
                    }
                }
                if buf.is_empty() {
                    let _ = pool_tx_reader.send(buf);
                    break;
                }
                // memchr::memrchr is SIMD-accelerated — slicing 4 MiB with
                // iter().rposition() showed up as noticeable CPU time.
                match memchr::memrchr(b'\n', &buf) {
                    Some(pos) => {
                        if pos + 1 < buf.len() {
                            carryover.extend_from_slice(&buf[pos + 1..]);
                            buf.truncate(pos + 1);
                        }
                    }
                    None => {
                        // Entire buffer is one unterminated line — keep
                        // accumulating. Return this buf empty to the pool
                        // first so we can grab a fresh one.
                        carryover.append(&mut buf);
                        let _ = pool_tx_reader.send(buf);
                        if eof {
                            break;
                        }
                        continue;
                    }
                }
                if work_txs[worker_idx]
                    .send(WorkMsg::Chunk(buf))
                    .is_err()
                {
                    return Ok(());
                }
                worker_idx = (worker_idx + 1) % num_workers;
                if eof {
                    break;
                }
            }
            if !carryover.is_empty() {
                if !carryover.ends_with(b"\n") {
                    carryover.push(b'\n');
                }
                let _ = work_txs[worker_idx].send(WorkMsg::Chunk(carryover));
            }
            // Signal every worker to exit.
            for tx in work_txs.into_iter() {
                let _ = tx.send(WorkMsg::End);
            }
            Ok(())
        })
    };

    // Main thread is the writer: reads results round-robin from worker
    // channels and writes them to disk in input-file order. The End
    // sentinels from each worker let us stop without tracking chunk totals.
    drop(pool_tx); // only reader + workers hold senders now
    let mut matched_lines_count: usize = 0;
    let mut workers_done = 0usize;
    let mut worker_idx = 0usize;
    while workers_done < num_workers {
        match result_rxs[worker_idx].recv() {
            Ok(ResultMsg::Chunk { bytes, matches }) => {
                output_stream.write_all(&bytes)?;
                matched_lines_count += matches;
                pb.set_position(bytes_read.load(Ordering::Relaxed));
            }
            Ok(ResultMsg::End) | Err(_) => {
                workers_done += 1;
            }
        }
        worker_idx = (worker_idx + 1) % num_workers;
    }
    output_stream.flush()?;

    if let Err(e) = reader_handle.join().expect("reader thread panicked") {
        eprintln!("Reader thread error: {}", e);
    }
    for h in worker_handles {
        h.join().expect("worker thread panicked");
    }

    pb.finish_and_clear();
    let elapsed = pb.elapsed().as_secs();
    println!(
        "Matched {} lines in file {} (took {})",
        matched_lines_count,
        args.input,
        format_elapsed(elapsed)
    );

    Ok(())
}
