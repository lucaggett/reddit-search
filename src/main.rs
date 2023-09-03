mod constants;
extern crate num_cpus;

use std::fs::File;
use std::path::PathBuf;
use zstd::Decoder;
use std::io::{BufRead, BufReader, Write, BufWriter};
use clap::{Arg, ArgAction, Command, value_parser};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::OpenOptions;
use std::string::String;
use constants::create_line_count_map;

fn process_line(line: &str, search_strings: &Vec<Vec<String>>) -> Option<String> {
    // switched this away from serde_json because it was very slow, and we don't need to parse the whole line
    if search_strings.iter().all(|formats| {
        formats.iter().any(|format| line.contains(format))
    }) {
        Some(line.clone().to_string())
    } else {
        None
    }
}

fn process_chunk(lines: Vec<String>, search_strings: &Vec<Vec<String>>) -> Vec<String> {
    lines.into_par_iter()
        .filter_map(|line| process_line(&line, search_strings))
        .collect()
}

fn main() -> std::io::Result<()> {
    let args = Command::new("reddit-search")
        .about("Utility to search the pushshift.io reddit dumps. Takes a zstd compressed file as input and outputs matching lines to a file.\nFor processing multiple files, use the --append option and loop over the directory using your shell of choice\n\nThe dumps are available here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee")
        .version(env!("CARGO_PKG_VERSION"))
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
                .help("Sets the output file to use.")
                .action(ArgAction::Set)
                .num_args(1)
                .default_value("reddit_comments.json"),
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
                .help("Sets the number of threads to use. Defaults to 4 cores or the number of logical cores on the system, whichever is lower.")
                .required(false)
                .num_args(1)
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize)),
        ).get_matches();



    // set the number of threads to 4 or the number of logical cores on the system, whichever is lower
    let mut threads = 4;
    if num_cpus::get() > 8 {
        threads = num_cpus::get();
    } else if args.contains_id("threads") {
        threads = *args.get_one::<usize>("threads").unwrap();
    }
    rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();

    // this is a magic number that seems to work well
    const CHUNK_SIZE: usize = 500_000;


    let input_path = args.get_one::<String>("input").unwrap().replace('\\', "/");
    if !input_path.ends_with(".zst") {
        let err_msg = format!("Input file must be a zstd compressed file. {} is not a zstd file.", input_path);
        eprintln!("{}", err_msg);
        return Ok(());
    }
    let input_buf = PathBuf::from(input_path.clone());
    let metadata = input_buf.metadata()?;
    let input_file = File::open(input_buf.clone())?;
    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    let output_path = args.get_one::<String>("output").unwrap();
    // if the output file exists, exit with an error mentioning the --append option
    if PathBuf::from(output_path).exists() && !args.contains_id("append") {
        let err_msg = format!("Output file {} already exists. Use the --append option to append to the file.", output_path);
        eprintln!("{}", err_msg);
    }
    let output_buf = PathBuf::from(output_path);
    let append_flag = *args.get_one("append").unwrap_or(&false);
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append_flag)
        .open(output_buf)?;
    let mut output_stream = BufWriter::new(output_file);
    let mut search_strings: Vec<Vec<String>> = Vec::new();
    for field in args.get_many::<String>("fields").unwrap() {
        let mut split = field.split(':');
        let field_key = split.next().unwrap().to_string();
        let value = split.next().unwrap().to_string();
        let formats = vec![
            format!("\"{}\":\"{}\"", field_key, value),
            format!("\"{}\":{}", field_key, value)
        ];
        search_strings.push(formats);
    }


    //println!("Starting reddit-search for {} ({} threads)", input_path, rayon::current_num_threads());

    let (tx, rx) = std::sync::mpsc::channel();

    rayon::spawn(move || {
        let mut chunk = Vec::with_capacity(CHUNK_SIZE);
        for line in input_stream.lines() {
            let line = line.expect("Failed to read line");
            chunk.push(line);

            if chunk.len() >= CHUNK_SIZE {
                tx.send(chunk).expect("Failed to send chunk");
                chunk = Vec::with_capacity(CHUNK_SIZE);
            }
        }

        if !chunk.is_empty() {
            tx.send(chunk).expect("Failed to send final chunk");
        }
    });

    let mut matched_lines_count = 0;
    let line_count_map = create_line_count_map();
    let file_name = input_path.split('/').last().unwrap();
    let mut num_lines = *line_count_map.get(file_name).unwrap_or(&0);
    if num_lines == 0 {
        println!("Warning: No line count found for {}. This will cause the progress percent to be inaccurate.", file_name);
        //estimate the number of lines as approximately 10,000,000 per GB
        let estimated_num_lines = (metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0;
        num_lines = estimated_num_lines as u64;
    }
    let pb = ProgressBar::new(num_lines);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})").expect("Failed to set progress bar style")
        .progress_chars("}=>-"));


    for chunk in rx {
        let matches = process_chunk(chunk, &search_strings);
        matched_lines_count += matches.len();

        for line in matches {
            writeln!(output_stream, "{}", line)?;
        }
        pb.inc(CHUNK_SIZE as u64)

    }
    pb.finish_and_clear();
    print!("Matched {} lines out of {} in file {}", matched_lines_count, num_lines, input_path);
    if pb.elapsed().as_secs() > 60 {
        if pb.elapsed().as_secs() > 120 {
            println!(" (took {} minutes, {} seconds)", pb.elapsed().as_secs() / 60, pb.elapsed().as_secs() % 60)
        } else {
            println!(" (took {} minute, {} seconds)", pb.elapsed().as_secs() / 60, pb.elapsed().as_secs() % 60)
        }
    } else {
        println!(" (took {} seconds)", pb.elapsed().as_secs());
    }

    Ok(())
}
