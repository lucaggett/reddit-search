# reddit-search
a tool for searching the pushshift reddit dumps written in rust. Available from crates.io via `cargo install reddit-search`

The dumps are available via torrent from here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee

a sample dump from january 2012 can be downloaded from IPFS at: https://ipfs.io/ipfs/QmTGmjq6jkXi7oVsT9vdMwNdnGimWUpB1YagpBRCZRDpdd?filename=RC_2012-01.zst

# usage
To see command line parameters, use reddit-search -h or --help

# Sample usage commands

## Basic Usage
```sh
reddit-search --input <input file path> --output <output file path> --fields <field:value> ...
```


## Iterating over the whole dataset

### Unix-like Shell (Bash, ZSH, etc)
```sh
for file in /path/to/dumps; do; reddit-search --append -i $file -o output.json -f field:value; done
```

### Powershell
```powershell
Get-ChildItem C:\path\to\dumps | ForEach-Object { reddit-search.exe --input $_.FullName -f subreddit:schwiiz -o output_schwiiz_submissions.json --append }
```
