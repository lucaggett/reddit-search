# reddit-search
a tool for searching the pushshift reddit dumps written in rust. Available from crates.io via `cargo install reddit-search`

The dumps are available via torrent from here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee

# usage
To see command line parameters, use reddit-search -h or --help

# Sample usage commands

## Getting all matching comments from the dataset
If you only want a subset of of comments (for example, all comments in /r/funny from 2012-01 to 2013-01), I recommend filtering the files passed into the program rather than filtering the JSON by date

### Unix-like Shell (Bash, ZSH, etc)
```sh
for file in /path/to/dumps; do; reddit-search --append -i $file -o output.json -f field:value; done
```

### Powershell
```powershell
Get-ChildItem C:\path\to\dumps | ForEach-Object { reddit-search.exe --input $_.FullName -f subreddit:schwiiz -o output_schwiiz_submissions.json --append }
```
