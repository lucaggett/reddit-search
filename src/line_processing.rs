use rayon::prelude::*;

pub(crate) fn process_line(line: &str, search_strings: &Vec<String>) -> Option<String> {
    // switched this away from serde_json because it was very slow, and we don't need to parse the whole line
    if search_strings
        .iter()
        .any(|search_string| line.to_lowercase().contains(search_string))
    {
        Some(line.to_string())
    } else {
        None
    }
}

pub(crate) fn process_chunk(lines: Vec<String>, search_strings: &Vec<String>) -> Vec<String> {
    lines
        .into_par_iter()
        .filter_map(|line| process_line(&line, search_strings))
        .collect()
}
