use std::fs::File;
use std::path::PathBuf;
use zstd::Decoder;
use std::io::{BufRead, BufReader, Write, BufWriter};
use clap::{command, Parser};
use std::collections::HashMap;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::fs::OpenOptions;

const CHUNK_SIZE: usize = 100000;
#[derive(Parser, Debug)]
#[command(name = "reddit-search")]
#[command(author = "Luc Aggett (luc@aggett.com)")]
#[command(version = "1.0")]
#[command(about = "utility to search and filter reddit dumps", long_about = None)]
struct Args {
    #[arg(short, long)]
    input: PathBuf,
    #[arg(short, long, default_value = "output.json")]
    output: String,
    #[arg(short, long, default_value = "20000")]
    chunk_size: usize,
    #[arg(num_args(1..), short, long)]
    fields: Vec<String>,
    #[arg(short, long, default_value = "false")]
    verbose: bool,
    #[arg(short, long, default_value = "false")]
    append: bool,

}

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
    // copy the input path so we can use it for a message later
    let input_path = args.input.clone();
    let input_file = File::open(args.input)?;
    let metadata = input_file.metadata()?;

    let mut decoder = Decoder::new(input_file)?;
    decoder.window_log_max(31)?;
    let input_stream = BufReader::new(decoder);

    let output_path = PathBuf::from(&args.output);
    let output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(args.append)
        .open(output_path.clone())?;
    let mut output_stream = BufWriter::new(output_file);
    //println!("{:?}", args.fields);
    let mut field_map: HashMap<String, String> = HashMap::new();
    for field in args.fields {
        let mut split = field.split(":");
        let field = split.next().unwrap().to_string();
        let value = split.next().unwrap().to_string();
        if field_map.contains_key(&field) {
            println!("{} already exists in field map, adding {} to existing value", field, value);
            let existing_value = field_map.get(&field.clone()).unwrap().to_string();
            field_map.insert(field.clone(), format!("{}|{}", existing_value, value));
        }
        else {
            field_map.insert(field, value);
        }
    }

    println!("Starting reddit-search for {} ({} threads)", input_path.display(), rayon::current_num_threads());

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

    // estimate the number of lines by multiplying the number of GB by 8000000 (This is an estimate I got from looking at a few sample files)
    let estimated_num_lines = ((metadata.len() as f64 / 1_000_000_000.0) * 10_000_000.0) as u64;
    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .template("[{elapsed_precise}] {pos} lines processed (~{msg}%)").expect("Failed to set progress bar style")
        .tick_chars("-/||\\-"));
    for chunk in rx {
        let matches = process_chunk(chunk, &field_map);
        matched_lines_count += matches.len();
        total_lines += CHUNK_SIZE;

        for line in matches {
            writeln!(output_stream, "{}", line)?;
        }
        if total_lines % 100_000 == 0 {
            pb.set_position(total_lines as u64); // Update progress bar with lines processed
            pb.set_message(format!("{:.2}", (total_lines as f64 / estimated_num_lines as f64) * 100.0));
        }
    }

    pb.finish_with_message("100");
    println!("Matched {} lines out of {}", matched_lines_count, total_lines);
    if matched_lines_count == 0 {
        println!("No matches found, deleting output file");
        std::fs::remove_file(output_path)?;
    }

    Ok(())
}
