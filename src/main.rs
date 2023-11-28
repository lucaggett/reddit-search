mod constants;
mod line_processing;
mod arguments;

extern crate num_cpus;

use crate::line_processing::process_chunk;
use std::fs::File;
use std::path::PathBuf;
use zstd::Decoder;
use std::io::{BufRead, BufReader, Write, BufWriter};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::OpenOptions;
use std::io;
use std::string::String;
use constants::create_line_count_map;
use crate::arguments::CommandLineArgs;


// this is mostly a utility function to get the number of lines in a file, used for creating the
// estimates used in the progress bar. I've left it in because it might be useful for something
// else in the future. Due to the bottleneck being the disk read speed, it'll take about the
// same time as using the program normally.
fn count_lines(file_name: &str) -> u64 {
    let input_buf = PathBuf::from(file_name);
    let metadata = input_buf.metadata().unwrap();
    let input_file = File::open(input_buf).unwrap();
    let mut decoder = Decoder::new(input_file).unwrap();
    decoder.window_log_max(31).unwrap();
    let input_stream = BufReader::new(decoder);
    let mut num_lines = 0;
    for _ in input_stream.lines() {
        num_lines += 1;
    }
    println!("{};{};{}", file_name, metadata.len(), num_lines);
    num_lines
}

fn main() -> std::io::Result<()> {
    let mut args = CommandLineArgs::new().unwrap();
    if args.linecount {
        count_lines(&args.input);
        return Ok(());
    }

    let search_fields: Vec<String>;
    if args.preset.is_some() {
        search_fields = arguments::get_preset_fields(&args.preset.unwrap()).unwrap();
    } else {
        let args_fields: Vec<&str> = args.fields.as_ref().unwrap().iter().map(|s| s.as_str()).collect();
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
        // if the value is an integer, a boolean or null do not add quotes
        if value.parse::<i64>().is_ok() || value == "true" || value == "false" || value == "null" {
            search_strings.push(format!("\"{}\":{}", field_key, value));
            continue;
        } else {
            // otherwise, add quotes
            search_strings.push(format!("\"{}\":\"{}\"", field_key, value));
        }
    }

    // this is a magic number that seems to work well
    const CHUNK_SIZE: usize = 100_000;

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
    let output_buf = PathBuf::from(args.output.clone());
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(args.append)
        .open(output_buf)?;

    // if the debug flag is set, print some general info
    if args.verbose {
        println!("Starting reddit-search for {} ({} threads) at {}", args.input, rayon::current_num_threads(), chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("Input file: {}", args.input);
        println!("Output file: {}", args.output);
        println!("Append: {}", args.append);
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
    let file_name = args.input.split('/').last().unwrap();
    let mut num_lines = *line_count_map.get(file_name).unwrap_or(&0);
    if num_lines == 0 {
        println!("Warning: No line count found for {}. This will cause the progress percent to be inaccurate.", file_name);
        // estimate the number of lines as approximately 10,000,000 per GB
        let estimated_num_lines = (metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0;
        num_lines = estimated_num_lines as u64;
    }
    let pb = ProgressBar::new(num_lines);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} | {percent}% | {eta} left").expect("Failed to set progress bar style")
        .progress_chars("=> "));


    for chunk in rx {
        let matches = process_chunk(chunk, &search_strings);
        matched_lines_count += matches.len();

        for line in matches {
            writeln!(output_stream, "{}", line)?;
        }
        pb.inc(CHUNK_SIZE as u64);
    }
    pb.finish_and_clear();
    print!("Matched {} lines out of {} in file {}", matched_lines_count, num_lines, args.input);
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

#[cfg(test)]
mod line_tests {
    use super::*;

    #[test]
    fn test_process_line_with_match() {
        let line = "This is a test line containing keyword.";
        let search_strings = vec!["keyword".to_string()];
        assert_eq!(process_line(line, &search_strings), Some(line.to_string()));
    }

    #[test]
    fn test_process_line_without_match() {
        let line = "This line does not contain the search string.";
        let search_strings = vec!["keyword".to_string()];
        assert_eq!(process_line(line, &search_strings), None);
    }

    #[test]
    fn test_process_line_case_insensitive() {
        let line = "This Line Contains KEYWORD in different case.";
        let search_strings = vec!["keyword".to_string()];
        assert_eq!(process_line(line, &search_strings), Some(line.to_string()));
    }

    #[test]
    fn test_process_line_empty_search_string() {
        let line = "This line will match with empty search string.";
        let search_strings = vec!["".to_string()];
        assert_eq!(process_line(line, &search_strings), Some(line.to_string()));
    }

    #[test]
    fn test_process_line_empty_line() {
        let line = "";
        let search_strings = vec!["keyword".to_string()];
        assert_eq!(process_line(line, &search_strings), None);
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::*;

    #[test]
    fn test_process_chunk_with_matches() {
        let lines = vec![
            "First line with keyword1.".to_string(),
            "Second line without.".to_string(),
            "Third line with keyword2.".to_string(),
        ];
        let search_strings = vec!["keyword1".to_string(), "keyword2".to_string()];
        let expected = vec![
            "First line with keyword1.".to_string(),
            "Third line with keyword2.".to_string(),
        ];
        assert_eq!(process_chunk(lines, &search_strings), expected);
    }

    #[test]
    fn test_process_chunk_without_matches() {
        let lines = vec![
            "First line.".to_string(),
            "Second line.".to_string(),
            "Third line.".to_string(),
        ];
        let search_strings = vec!["nonexistent".to_string()];
        assert_eq!(process_chunk(lines, &search_strings), Vec::<String>::new());
    }
}
