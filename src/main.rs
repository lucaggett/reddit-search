mod arguments;
mod constants;
mod line_processing;

extern crate num_cpus;

use crate::arguments::CommandLineArgs;
use crate::line_processing::process_chunk;
use constants::create_line_count_map;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{ParallelBridge, ParallelIterator};
use rayon::ThreadPoolBuilder;
use std::fs::File;
use std::fs::OpenOptions;
use std::io;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::string::String;
use zstd::Decoder;

// this is mostly a utility function to get the number of lines in a file, used for creating the
// estimates used in the progress bar. I've left it in because it might be useful for something
// else in the future. Due to the bottleneck being the disk read speed, it'll take about the
// same time as using the program normally.
fn count_lines(file_name: &str) -> () {
    let input_buf = PathBuf::from(file_name);
    let metadata = input_buf.metadata().unwrap();
    let input_file = File::open(input_buf).unwrap();
    let mut decoder = Decoder::new(input_file).unwrap();
    decoder.window_log_max(31).unwrap();
    let input_stream = BufReader::new(decoder);
    let num_lines = input_stream.lines().count();

    println!("{};{};{}", file_name, metadata.len(), num_lines);
}

fn get_output_stream(output_file: &str, append: bool) -> io::Result<BufWriter<File>> {
    let output_buf = PathBuf::from(output_file);
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(append)
        .open(output_buf)?;
    let output_stream = BufWriter::new(output_file);
    Ok(output_stream)
}

fn main() -> std::io::Result<()> {
    let mut args = CommandLineArgs::new().unwrap();

    // set the number of threads to use
    ThreadPoolBuilder::new()
        .num_threads(args.threads)
        .build_global()
        .unwrap();

    if args.linecount {
        count_lines(&args.input);
        return Ok(());
    }

    let search_fields: Vec<String>;
    if args.preset.is_some() {
        search_fields = arguments::get_preset_fields(&args.preset.unwrap()).unwrap();
    } else {
        let args_fields: Vec<&str> = args
            .fields
            .as_ref()
            .unwrap()
            .iter()
            .map(|s| s.as_str())
            .collect();
        search_fields = args_fields
            .iter()
            .map(|s| s.to_string().to_lowercase())
            .collect();
    }

    let mut search_strings: Vec<String> = Vec::new();
    for field in search_fields {
        let mut split = field.split(':');
        // test if split contains two elements
        if split.clone().count() != 2 {
            let err_msg = format!("Field {} is not in the format <field>:<value>", field);
            eprintln!("{}", err_msg);
            return Ok(());
        }
        let field_key = split.next().unwrap().to_string();
        let value = split.next().unwrap().to_string();
        // if the value is an integer, a boolean or null do not add quotes
        if value.parse::<i64>().is_ok() || value == "true" || value == "false" || value == "null" {
            if args.input.starts_with("RC_2023") {
                search_strings.push(format!("\"{}\": {}", field_key, value));
            } else {
                search_strings.push(format!("\"{}\":{}", field_key, value));
            }
            continue;
        } else {
            // otherwise, add quotes
            if args.input.starts_with("RC_2023") {
                search_strings.push(format!("\"{}\": \"{}\"", field_key, value));
            } else {
                search_strings.push(format!("\"{}\":\"{}\"", field_key, value));
            }
        }
    }

    let input_buf = PathBuf::from(args.input.clone());
    let metadata = input_buf.metadata()?;
    // check if input file exists and is a file
    if !metadata.is_file() {
        let err_msg = format!("Input file {} does not exist.", args.input);
        eprintln!("{}", err_msg);
        return Ok(());
    }

    let input_file = File::open(input_buf)?;
    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    if PathBuf::from(args.output.clone()).exists() && !args.append && !args.overwrite {
        eprint!("File {} already exists. Enter 'a' to append to the file, 'o' to overwrite, or anything else to exit: ", args.output.clone());
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        user_input = user_input.trim().to_string();
        if user_input == "a" {
            args.append = true;
        } else if user_input == "o" {
            args.append = false;
        } else {
            println!("Exiting");
            return Ok(());
        }
    }
    // if the debug flag is set, print some general info
    if args.verbose {
        println!(
            "Starting reddit-search for {} ({} threads) at {}",
            args.input,
            rayon::current_num_threads(),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );
        println!("Input file: {}", args.input);
        println!("Output file: {}", args.output);
        println!("Append: {}", args.append);
        println!("Threads: {}", rayon::current_num_threads());
        println!("Line count: {}", metadata.len());
        println!("Search fields: {}", search_strings.join(", "));
        println!("Chunk size: {}", args.chunk_size);
    }

    //let mut matched_lines_count = 0;
    let line_count_map = create_line_count_map();
    let file_name = args.input.split('/').last().unwrap();
    let mut num_lines = *line_count_map.get(file_name).unwrap_or(&0);
    if num_lines == 0 {
        println!("Warning: No line count found for {}. This will cause the progress percent to be inaccurate.", file_name);
        // estimate the number of lines as approximately 10,000,000 per GB
        let estimated_num_lines = (metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0;
        num_lines = estimated_num_lines as u64;
    }
    let pb = ProgressBar::new(num_lines);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {human_pos}/{human_len} | {percent}% | {eta} left",
            )
            .expect("Failed to set progress bar style")
            .progress_chars("=> "),
    );

    let (tx, rx) = std::sync::mpsc::channel();

    // spawn threads to read the input file and send chunks to the main thread
    rayon::spawn(move || {
        let mut chunk = Vec::with_capacity(args.chunk_size);
        for line in input_stream.lines() {
            let line = line.expect("Failed to read line");
            chunk.push(line);

            if chunk.len() >= args.chunk_size {
                tx.send(chunk).expect("Failed to send chunk");
                chunk = Vec::with_capacity(args.chunk_size);
            }
        }

        if !chunk.is_empty() {
            tx.send(chunk).expect("Failed to send final chunk");
        }
    });
    // Refactored for parallel processing on decoding
    let matched_lines_count: usize = rx
        .into_iter()
        .par_bridge()
        .map(|chunk| {
            let mut output_stream =
                get_output_stream(&args.output, args.append).expect("Failed to get output stream");
            let matches = process_chunk(chunk, &search_strings);
            let matched_lines_count = matches.len();
            for line in matches {
                writeln!(output_stream, "{}", line).expect("Failed to write line");
            }
            pb.inc(args.chunk_size as u64);
            matched_lines_count
        })
        .sum();

    pb.finish_and_clear();
    print!(
        "Matched {} lines out of {} in file {}",
        matched_lines_count, num_lines, args.input
    );
    if pb.elapsed().as_secs() > 60 {
        if pb.elapsed().as_secs() > 120 {
            println!(
                " (took {} minutes, {} seconds)",
                pb.elapsed().as_secs() / 60,
                pb.elapsed().as_secs() % 60
            )
        } else {
            println!(
                " (took {} minute, {} seconds)",
                pb.elapsed().as_secs() / 60,
                pb.elapsed().as_secs() % 60
            )
        }
    } else {
        println!(" (took {} seconds)", pb.elapsed().as_secs());
    }

    Ok(())
}
