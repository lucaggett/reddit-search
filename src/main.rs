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
use std::io;
use std::string::String;
use constants::create_line_count_map;

fn process_line(line: &str, search_strings: &Vec<String>) -> Option<String> {
    // switched this away from serde_json because it was very slow, and we don't need to parse the whole line
    if search_strings.iter().any(|formats| { line.to_lowercase().contains(&formats.to_lowercase()) }) {
        Some(line.to_string())
    } else {
        None
    }
}

fn process_chunk(lines: Vec<String>, search_strings: &Vec<String>) -> Vec<String> {
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
            .required_unless_present("preset")
            .action(ArgAction::Set)
            .value_parser(value_parser!(String))
            .num_args(1..)
        )
        .arg(Arg::new("append")
                 .short('a')
                 .long("append")
                 .help("Append to the output file instead of overwriting it.")
                 .required(false)
                 .conflicts_with("overwrite")
                 .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("overwrite")
                 .short('w')
                 .long("overwrite")
                 .help("Overwrite the output file instead of appending to it.")
                 .required(false)
                 .conflicts_with("append")
                 .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("threads")
                 .short('t')
                 .long("threads")
                 .value_name("THREADS")
                 .help("Sets the number of threads to use. Defaults to the number of logical cores on the system.")
                 .required(false)
                 .num_args(1)
                 .action(ArgAction::Set)
                 .value_parser(value_parser!(usize)),
        )
        .arg(Arg::new("random")
                 .short('r')
                 .long("random")
                 .help("Randomly sample the input file. Useful for testing.")
                 .required(false)
                 .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("linecount")
                 .short('l')
                 .long("linecount")
                 .help("Print the number of lines in the input file and exit.")
                 .required(false)
                 .action(ArgAction::SetTrue),
        )
        .arg(Arg::new("preset")
            .short('p')
            .long("preset")
            .value_name("PRESET")
            .help("Use a preset instead of specifying fields manually. Available presets are: en_news, en_politics, en_hate_speech")
            .required_unless_present("fields")
            .action(ArgAction::Set)
            .num_args(1)
        )
        .arg(Arg::new("verbose")
                 .short('v')
                 .long("verbose")
                 .help("Print verbose output.")
                 .required(false)
                 .action(ArgAction::SetTrue),
        ).get_matches();

    // set the number of threads if "threads" argument is provided
    let threads_flag: usize = *args.get_one("threads").unwrap_or(&0);
    if threads_flag > 0 {
        let threads = *args.get_one::<usize>("threads").unwrap();
        rayon::ThreadPoolBuilder::new().num_threads(threads).build_global().unwrap();
    }
    // this is mostly a utility function to get the number of lines in a file, used for creating the
    // estimates used in the progress bar. I've left it in because it might be useful for something
    // else in the future. Due to the bottleneck being the disk read speed, it'll take about the
    // same time as using the program normally.
    let linecount_flag: bool = *args.get_one("linecount").unwrap_or(&false);
    if linecount_flag {
        let input_path = args.get_one::<String>("input").unwrap().replace('\\', "/");
        let file_name = input_path.split('/').last().unwrap();
        let input_buf = PathBuf::from(input_path.clone());
        let metadata = input_buf.metadata()?;
        let input_file = File::open(input_buf.clone())?;
        let mut decoder = Decoder::new(input_file)?;
        decoder.window_log_max(31)?;
        let input_stream = BufReader::new(decoder);
        let mut num_lines = 0;
        for _ in input_stream.lines() {
            num_lines += 1;
        }
        println!("{};{};{}", file_name, metadata.len(), num_lines);
    }

    // Search fields
    // TESTING FOR SUBCOMMAND "preset"
    let search_fields: Vec<String>;
    let preset = args.get_one::<String>("preset");
    if preset.is_some() {
        let preset = preset.unwrap();
        let presets_map = constants::get_presets();
        // check if the preset exists
        if !presets_map.contains_key(preset.as_str()) {
            let err_msg = format!("Preset {} not found. Available presets are: {}", preset, presets_map.keys().map(|s| s.to_string()).collect::<Vec<String>>().join(", "));
            eprintln!("{}", err_msg);
            return Ok(());
        }
        // preset map contains a list of strings, so we need to convert the preset string to a &str
        let args_fields = presets_map.get(preset.as_str()).unwrap().to_vec();
        // convert all values in the vec from &str to String
        search_fields = args_fields.iter().map(|s| s.to_string()).collect();
    } else {
        let args_fields: Vec<&str> = args.get_many::<String>("fields").unwrap().map(|s| s.as_str()).collect();
        println!("{:?}", args_fields);
        search_fields = args_fields.iter().map(|s| s.to_string()).collect();
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
        search_strings.push(format!("\"{}\":\"{}\"", field_key, value));
        search_strings.push(format!("\"{}\":{}", field_key, value));
    }

    // this is a magic number that seems to work well
    const CHUNK_SIZE: usize = 500_000;

    // fix windows-style paths
    let input_path = args.get_one::<String>("input").unwrap().replace('\\', "/");
    // check that the input file is a zstd file
    if !input_path.ends_with(".zst") {
        let err_msg = format!("Input file must be a zstd compressed file. {} is not a zstd file.", input_path);
        eprintln!("{}", err_msg);
        return Ok(());
    }
    let input_buf = PathBuf::from(input_path.clone());
    let metadata = input_buf.metadata()?;
    // check if input file exists and is a file
    if !metadata.is_file() {
        let err_msg = format!("Input file {} does not exist.", input_path);
        eprintln!("{}", err_msg);
        return Ok(());
    }

    let input_file = File::open(input_buf.clone())?;
    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    let output_path = args.get_one::<String>("output").unwrap();
    // if the output file exists, exit with an error mentioning the --append option
    let mut append_flag = *args.get_one("append").unwrap_or(&false);
    let overwrite_flag = *args.get_one("overwrite").unwrap_or(&false);
    if PathBuf::from(output_path).exists() && !append_flag && !overwrite_flag {
        //let err_msg = format!("Output file {} already exists. Use --append or --overwrite", output_path);
        //eprintln!("{}", err_msg);
        eprint!("Enter 'a' to append to the file, 'o' to overwrite, or anything else to exit: ");
        let mut user_input = String::new();
        io::stdin()
            .read_line(&mut user_input)
            .expect("Failed to read line");
        user_input = user_input.trim().to_string();
        if user_input == "a" {
            append_flag = true;
        } else if user_input == "o" {
            append_flag = false;
        } else {
            println!("Exiting");
            return Ok(());
        }
    }
    let output_buf = PathBuf::from(output_path);
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(append_flag)
        .open(output_buf)?;

    // if the debug flag is set, print some general info
    let verbose_flag = *args.get_one("verbose").unwrap_or(&false);
    if verbose_flag {
        println!("Starting reddit-search for {} ({} threads) at {}", input_path, rayon::current_num_threads(), chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("Input file: {}", input_path);
        println!("Output file: {}", output_path);
        println!("Append: {}", append_flag);
        println!("Threads: {}", rayon::current_num_threads());
        println!("Line count: {}", metadata.len());
        println!("Search fields: {}", search_strings.join(", "));
    }

    let mut output_stream = BufWriter::new(output_file);
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
        // estimate the number of lines as approximately 10,000,000 per GB
        let estimated_num_lines = (metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0;
        num_lines = estimated_num_lines as u64;
    }
    let pb = ProgressBar::new(num_lines);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})").expect("Failed to set progress bar style")
        .progress_chars("#>-"));


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
