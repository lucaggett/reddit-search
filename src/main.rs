use std::fs::File;
use std::path::PathBuf;
use zstd::Decoder;
use std::io::{BufRead, BufReader, Write, BufWriter};
use clap::{Arg, ArgAction, Command};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::OpenOptions;


fn process_line(line: String, search_strings: &Vec<Vec<String>>) -> Option<String> {
    if search_strings.iter().all(|formats| {
        formats.iter().any(|format| line.contains(format))
    }) {
        Some(line)
    } else {
        None
    }
}

fn process_chunk(lines: Vec<String>, search_strings: &Vec<Vec<String>>) -> Vec<String> {
    lines.into_par_iter()
        .filter_map(|line| process_line(line, &search_strings))
        .collect()
}

fn main() -> std::io::Result<()> {
    let args = Command::new("reddit-search")
        .about("Utility to search the pushshift.io reddit dumps. Takes a zstd compressed file as input and outputs matching lines to a file.\nFor processing multiple files, use the --append option and loop over the directory using your shell of choice\n\nThe dumps are available here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee")
        .version("0.2.0")
        .author("Luc Aggett (luc@aggett.com")
        .arg_required_else_help(true)
        .arg(Arg::new("input")
                .short('i')
                .long("input")
                .value_name("INPUT")
                .help("Sets the input file to use. Must be a zstd compressed newline delimited JSON file.")
                .required(true)
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(Arg::new("output")
                .short('o')
                .long("output")
                .value_name("OUTPUT")
                .help("Sets the output file to use. Will be created if it doesn't exist.")
                .required(true)
                .action(ArgAction::Set)
                .num_args(1),
        )
        .arg(Arg::new("fields")
                .short('f')
                .long("fields")
                .value_name("FIELDS")
                .help("Sets the fields to search. Must be in the format <field>:<value>. Can be specified multiple times.")
                .required(true)
                .action(ArgAction::Set)
                .num_args(1..)
        )
        .arg(Arg::new("append")
                .short('a')
                .long("append")
                .help("Append to the output file instead of overwriting it.")
                .required(false)
                .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("THREADS")
                .help("Sets the number of threads to use. Defaults to the number of logical cores on the system.")
                .required(false)
                .num_args(1)
                .action(ArgAction::Set),
        )
        .arg(Arg::new("chunk_size")
                .short('c')
                .long("chunk-size")
                .value_name("chunk_size")
                .help("Sets the number of lines to process in each chunk. Defaults to 500,000.")
                .required(false)
                .num_args(1)
                .action(ArgAction::Set),
        ).get_matches();

    // copy the input path so we can use it for a message later
    if args.contains_id("threads") {
        let threads = args.get_one::<usize>("threads").unwrap().clone();
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().expect("Failed to set thread pool size");
    }

    let mut chunk_size: usize = 500_000;
    if args.contains_id("chunk_size") {
        chunk_size = args.get_one::<usize>("chunk_size").unwrap().clone();
    }


    let input_path = args.get_one::<String>("input").unwrap();
    let input_buf = PathBuf::from(input_path);
    let input_file = File::open(input_buf.clone())?;
    let metadata = input_buf.metadata()?;
    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    let output_path = args.get_one::<String>("output").unwrap();
    let output_buf = PathBuf::from(output_path);
    let append_flag = args.get_one("append").unwrap_or(&false).clone();
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append_flag)
        .open(output_buf)?;
    let mut output_stream = BufWriter::new(output_file);
    let mut search_strings: Vec<Vec<String>> = Vec::new();
    for field in args.get_many::<String>("fields").unwrap() {
        let mut split = field.split(":");
        let field_key = split.next().unwrap().to_string();
        let value = split.next().unwrap().to_string();
        let formats = vec![
            format!("\"{}\":\"{}\"", field_key, value),
            format!("\"{}\":{}", field_key, value)
        ];
        search_strings.push(formats);
    }


    println!("Starting reddit-search for {} ({} threads)", input_path, rayon::current_num_threads());

    let (tx, rx) = std::sync::mpsc::channel();

    rayon::spawn(move || {
        let mut chunk = Vec::with_capacity(chunk_size);
        for line in input_stream.lines() {
            let line = line.expect("Failed to read line");
            chunk.push(line);

            if chunk.len() >= chunk_size {
                tx.send(chunk).expect("Failed to send chunk");
                chunk = Vec::with_capacity(chunk_size);
            }
        }

        if !chunk.is_empty() {
            tx.send(chunk).expect("Failed to send final chunk");
        }
    });

    let mut matched_lines_count = 0;
    let mut total_lines = 0;

    // estimate the number of lines by multiplying the number of GB by 8000000 (This is an estimate I got from looking at a few sample files)
    let estimated_num_lines = ((metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0) as u64;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("[{elapsed_precise}] {pos} lines processed ({msg})").expect("Failed to set progress bar style")
        .tick_chars("-/||\\-"));


    for chunk in rx {
        let matches = process_chunk(chunk, &search_strings);
        matched_lines_count += matches.len();
        total_lines += chunk_size;

        for line in matches {
            writeln!(output_stream, "{}", line)?;
        }

        pb.set_position(total_lines as u64); // Update progress bar with lines processed
        let percent = (total_lines as f64 / estimated_num_lines as f64) * 100.0;
        if percent < 100.0 {
            pb.set_message(format!("~{:.0}%", percent));
        } else {
            pb.set_message("Please wait...");
        }

    }

    pb.finish_with_message("Done!");
    println!("Matched {} lines out of {}", matched_lines_count, total_lines);
    if matched_lines_count == 0 && !append_flag {
        println!("No matches found, deleting output file");
        std::fs::remove_file(output_path)?;
    }

    Ok(())
}
