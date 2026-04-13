use std::collections::HashMap;

// the presets are a hashmap of strings (preset names) mapping to vectors of values (the field strings to include)
pub(crate) fn get_presets() -> HashMap<&'static str, Vec<&'static str>> {
    HashMap::from([
        ("en_news", vec!["subreddit:news", "subreddit:worldnews", "subreddit:inthenews", "subreddit:worldpolitics"]),
        ("en_politics", vec!["subreddit:politics", "subreddit:PoliticalDiscussion", "subreddit:geopolitics", "subreddit:NeutralPolitics", "subreddit:Ask_Politics", "subreddit:PoliticalHumor", "subreddit:PoliticalCompassMemes", "subreddit:PoliticalMemes", "subreddit:PoliticalDiscussion", "subreddit:ShitPoliticsSay"]),
        ("en_science", vec!["subreddit:science", "subreddit:EverythingScience", "subreddit:AskScience", "subreddit:EverythingScience"]),
        ("en_hate_speech", vec!["subreddit:Physical_Removal", "subreddit:MillionDollarExtreme", "subreddit:GasTheKikes", "subreddit:FatPeopleHate", "subreddit:Beatingwomen", "subreddit:niggers", "subreddit:UncensoredNews"]),
        ("controversial", vec!["controversiality:1"]),
        ("qa_subreddits", vec!["subreddit:AskReddit", "subreddit:explainlikeimfive", "subreddit:AskScience", "subreddit:AskHistorians", "subreddit:techsupport", "subreddit:legaladvice"]),
    ])
}
