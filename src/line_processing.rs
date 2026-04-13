use aho_corasick::AhoCorasick;

// Process a chunk of raw decompressed bytes in-place, using caller-provided
// reusable scratch and output buffers. The chunk is guaranteed to end on a
// newline boundary by the reader so split-on-'\n' gives whole lines.
//
// `scratch` holds the ascii-lowercased copy of the current line (reused).
// `out` accumulates matched lines terminated by '\n' (reused across calls —
// the caller is responsible for clearing it before the call).
//
// Returns the number of matched lines.
pub(crate) fn process_chunk(
    chunk: &[u8],
    ac: &AhoCorasick,
    scratch: &mut Vec<u8>,
    out: &mut Vec<u8>,
) -> usize {
    let mut count = 0usize;
    for raw in chunk.split(|&b| b == b'\n') {
        if raw.is_empty() {
            continue;
        }
        let line: &[u8] = if raw.last() == Some(&b'\r') {
            &raw[..raw.len() - 1]
        } else {
            raw
        };
        if line.is_empty() {
            continue;
        }
        scratch.clear();
        scratch.extend_from_slice(line);
        scratch.make_ascii_lowercase();
        if ac.is_match(&scratch[..]) {
            out.extend_from_slice(line);
            out.push(b'\n');
            count += 1;
        }
    }
    count
}
