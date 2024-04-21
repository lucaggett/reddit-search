# reddit-search
a tool for searching the pushshift reddit dumps written in rust. Available from crates.io via `cargo install reddit-search`

if you do not have cargo or rustc, please follow the steps outlined in the official rust documentation:
https://www.rust-lang.org/tools/install

The dumps are available via torrent from here: https://academictorrents.com/details/7c0645c94321311bb05bd879ddee4d0eba08aaee

# usage
To see command line parameters, use reddit-search -h or --help

# Sample usage commands

## Basic Usage
```sh
reddit-search --input <input file path> --output <output file path> --fields <field:value> ...
```

## Presets

| Preset Name       | Description                                                              |
|-------------------|--------------------------------------------------------------------------|
| `en_news`         | Subreddits focused on global and regional news and current events.       |
| `en_politics`     | A range of subreddits covering various political discussions, humor, and memes, including general politics and specific political orientations. |
| `en_science`      | Subreddits dedicated to general science, scientific inquiries, and discussions on scientific advancements. |
| `en_hate_speech`  | Subreddits known for promoting hate speech and controversial content.    |
| `controversial`   | Content with high levels of controversy across various themes.           |

Each preset is a collection of filters designed to target specific themes. Should you be interested in using this and would like additional filters to be added, do not hesitate to contact me.


# Descriptions of the fields contained within reddit dumps

Note that not all data contains all of these fields (for example, a comment from 2007 would not have the "gilded" field since that system was not implemented until later.)

Boolean values are saved numerically (0 is false, 1 is true)

| Field                   | Description |
|-------------------------|-------------|
| archived                | Boolean indicating if the item is archived |
| id                      | Unique identifier of the item |
| controversiality        | Boolean indicating if the item is controversial |
| body                    | Text content of the item |
| ups                     | Number of upvotes |
| score_hidden            | Boolean indicating if the score is hidden |
| edited                  | Boolean indicating if the item has been edited |
| distinguished           | Status of the item (e.g., null, moderator) |
| created_utc             | UTC timestamp of item creation |
| name                    | Another unique identifier (?) |
| gilded                  | Number indicating how many times the item was gilded |
| score                   | Total score of the item |
| subreddit_id            | Identifier of the subreddit |
| link_id                 | Identifier of the link to the comment |
| author_flair_text       | Text of the author's flair |
| subreddit               | Name of the subreddit |
| retrieved_on            | UTC timestamp of when the item was retrieved |
| parent_id               | Identifier of the parent item |
| downs                   | Number of downvotes |
| author_flair_css_class  | CSS class of the author's flair |
| author                  | Name of the author |


# Versioning

Older versions of the program can be downloaded using version overrides with cargo. Tags are not carried over to github.

## Disclaimer

"reddit-search" is an independent open-source tool designed for searching Zstandard (zst) dumps from Reddit. It is not affiliated, associated, authorized, endorsed by, or in any way officially connected with Reddit, Inc. The use of the Reddit name and any related trademarks in this project is purely for descriptive purposes. The trademarks and product names belong to their respective owners, who are not affiliated with, do not endorse, and do not sponsor "reddit-search". This project is developed under the principles of fair use and open source collaboration.

