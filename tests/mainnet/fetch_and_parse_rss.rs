
use reqwest;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
struct RssChannel {
    title: String,
    #[serde(rename = "item")]
    items: Vec<RssItem>,
}

#[derive(Debug, Deserialize)]
struct RssItem {
    title: String,
    link: String,
    pub_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RssFeed {
    channel: RssChannel,
}

async fn fetch_rss_feed(url: &str) -> Result<RssFeed, Box<dyn Error>> {
    let response = reqwest::get(url).await?.text().await?;
    let feed: RssFeed = from_str(&response)?;
    Ok(feed)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let feed_url = "https://example.com/feed.rss";
    let feed = fetch_rss_feed(feed_url).await?;

    println!("Feed Title: {}", feed.channel.title);
    println!("Latest Items:");
    for item in feed.channel.items.iter().take(5) {
        println!("- {}", item.title);
        if let Some(date) = &item.pub_date {
            println!("  Published: {}", date);
        }
        println!("  Link: {}", item.link);
    }

    Ok(())
}