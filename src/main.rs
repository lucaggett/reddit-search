use std::fs::File;
use std::path::PathBuf;
use zstd::Decoder;
use std::io::{BufRead, BufReader, Write, BufWriter};
use clap::{command, Parser};
use std::collections::HashMap;
use std::thread::sleep;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "reddit-search")]
#[command(author = "Luc Aggett (luc@aggett.com)")]
#[command(version = "1.0")]
#[command(about = "utility to search and filter reddit dumps", long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long, default_value = "output.json")]
    output: PathBuf,
    #[arg(num_args(0..))]
    fields: Vec<String>,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

const CHUNK_SIZE: usize = 10_000; // Adjust based on experimentation

fn process_line(line: String, field_map: &HashMap<String, String>) -> Option<String> {
    if field_map.iter().all(|(field, value)| {
        // If the line contains the field and value in the format "field":"value" or "field":value, then return the line
        line.contains(&format!("\"{}\":\"{}\"", field, value)) || line.contains(&format!("\"{}\":{}", field, value))
    }) {
        Some(line)
    } else {
        None
    }
}

fn process_chunk(lines: Vec<String>, field_map: &HashMap<String, String>) -> Vec<String> {
    lines.into_par_iter()
        .filter_map(|line| process_line(line, field_map))
        .collect()
}

fn main() -> std::io::Result<()> {
    let args = Args::parse();
    // copy the input path so we can use it in the progress bar
    let input_path = args.input.clone().into_os_string().into_string().unwrap();
    let input_file = File::open(args.input)?;

    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    let output_file = File::create(args.output)?;
    let mut output_stream = BufWriter::new(output_file);


    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("[{elapsed}] Processing {spinner} {pos} lines").expect("Failed to set progress bar style")
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "));


    let field_map: HashMap<String, String> = args.fields.iter().filter_map(|field| {
        let parts: Vec<&str> = field.split(':').collect();
        if parts.len() == 2 {
            Some((parts[0].to_string(), parts[1].to_string()))
        } else {
            None
        }
    }).collect();

    if field_map.len() == 1 {
        println!("Fetching all comments with {} = {} from {}", field_map.keys().next().unwrap(), field_map.values().next().unwrap(), input_path);
    } else {
        println!("Fetching all comments with the following fields from {}", input_path);
        for (field, value) in field_map.iter() {
            println!("{} = {}", field, value);
        }
    }
    sleep(std::time::Duration::from_millis(2000));
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
    let mut total_lines = 0;

    for chunk in rx {
        let matches = process_chunk(chunk, &field_map);
        matched_lines_count += matches.len();
        total_lines += CHUNK_SIZE;

        for line in matches {
            writeln!(output_stream, "{}", line)?;
        }
        if total_lines % 250_000 == 0 {
            pb.set_position(total_lines as u64); // Update progress bar with lines processed
        }
    }

    pb.finish_with_message("done");
    println!("Matched {} lines out of {}", matched_lines_count, total_lines);

    Ok(())
}
