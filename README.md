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
Get-ChildItem C:\path\to\dumps | ForEach-Object { reddit-search.exe --input $_.FullName -f KEY:DATA -o OUTPUT_FILENAME.json --append }
```

| Field                   | Description |
|-------------------------|-------------|
| archived                | Boolean indicating if the item is archived |
| id                      | Unique identifier of the item |
| controversiality        | Numeric value representing the controversiality |
| body                    | Text content of the item |
| ups                     | Number of upvotes |
| score_hidden            | Boolean indicating if the score is hidden |
| edited                  | Boolean indicating if the item has been edited |
| distinguished           | Status of the item (e.g., null, moderator) |
| created_utc             | UTC timestamp of item creation |
| name                    | Name of the item |
| gilded                  | Number indicating how many times the item was gilded |
| score                   | Total score of the item |
| subreddit_id            | Identifier of the subreddit |
| link_id                 | Identifier of the linked item |
| author_flair_text       | Text of the author's flair |
| subreddit               | Name of the subreddit |
| retrieved_on            | UTC timestamp of when the item was retrieved |
| parent_id               | Identifier of the parent item |
| downs                   | Number of downvotes |
| author_flair_css_class  | CSS class of the author's flair |
| author                  | Name of the author |
