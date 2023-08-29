# reddit-search
a tool for searching the pushshift reddit dumps written in rust

# usage
To see command line parameters, use reddit-search -h

# Sample usage commands

## Unix-like Shell (Bash, ZSH, etc)
```sh
for file in /path/to/dumps; do; reddit-search --append -i $file -o output.json -f field:value; done
```

## Powershell
```powershell
```
