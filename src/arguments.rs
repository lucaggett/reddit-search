extern crate clap;

use crate::constants::get_presets;
use clap::{value_parser, Arg, ArgAction, Command};

pub fn get_preset_fields(preset: &str) -> Option<Vec<String>> {
    let presets_map = get_presets();
    // check if the preset exists
    if !presets_map.contains_key(preset) {
        let err_msg = format!(
            "Preset {} not found. Available presets are: {}",
            preset,
            presets_map
                .keys()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        );
        eprintln!("{}", err_msg);
        return None;
    }
    // preset map contains a list of strings, so we need to convert the preset string to a &str
    let args_fields = presets_map.get(preset).unwrap().to_vec();
    // convert all values in the vec from &str to String
    let search_fields = args_fields.iter().map(|s| s.to_string()).collect();
    return Some(search_fields);
}

pub struct CommandLineArgs {
    pub input: String,
    pub output: String,
    pub fields: Option<Vec<String>>,
    pub append: bool,
    pub chunk_size: usize,
    pub overwrite: bool,
    pub random: bool,
    pub linecount: bool,
    pub preset: Option<String>,
    pub verbose: bool,
    pub threads: usize,
}

impl CommandLineArgs {
    pub fn new() -> Result<Self, String> {
        let args = Command::new("reddit-search")
            .about("Utility to search the pushshift.io reddit dumps. Takes a zstd compressed file as input and outputs matching lines to a file. \n\nThe dumps are available here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee\n\nNote: Due to performance constraints, backreferences and lookaround assertions are not supported.")
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
                .help("Sets the fields to search. Must be in the format <field>:<regex>. Can be specified multiple times.")
                .required_unless_present("preset")
                .required_unless_present("linecount")
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
                .required_unless_present("linecount")
                .action(ArgAction::Set)
                .num_args(1)
            )
            .arg(Arg::new("chunk-size")
                     .short('c')
                     .long("chunk-size")
                     .value_name("CHUNK_SIZE")
                     .help("Sets the chunk size to use when searching. Defaults to 100,000.")
                     .required(false)
                     .action(ArgAction::Set)
                     .value_parser(value_parser!(usize))
                     .default_value("100000"),
            )
            .arg(Arg::new("verbose")
                     .short('v')
                     .long("verbose")
                     .help("Print verbose output.")
                     .required(false)
                     .action(ArgAction::SetTrue),
            )
            .arg(Arg::new("threads")
                .short('t')
                .long("threads")
                .value_name("THREADS")
                .help("Sets the number of threads to use. Defaults to half the number of logical cores or 3, whichever is lower.")
                .required(false)
                .action(ArgAction::Set)
                .value_parser(value_parser!(usize)))
            .get_matches();

        // Extract values from args
        Ok(Self {
            input: args
                .get_one::<String>("input")
                .ok_or("Failed to parse input path, double check the arguments")?
                .replace("\\", "/"),
            output: args
                .get_one::<String>("output")
                .ok_or("Failed to parse output path, double check the arguments")?
                .replace("\\", "/"),
            fields: Some(
                args.get_many::<String>("fields")
                    .map_or_else(Vec::new, |values| values.map(ToString::to_string).collect()),
            ),
            append: *args.get_one("append").unwrap_or(&false),
            overwrite: *args.get_one("overwrite").unwrap_or(&false),
            random: *args.get_one("random").unwrap_or(&false),
            chunk_size: *args.get_one("chunk-size").unwrap_or(&100_000),
            linecount: *args.get_one("linecount").unwrap_or(&false),
            preset: args.get_one::<String>("preset").cloned(),
            verbose: *args.get_one("verbose").unwrap_or(&false),
            threads: *args.get_one::<usize>("threads").unwrap_or(&num_cpus::get()),
        })
    }
}
