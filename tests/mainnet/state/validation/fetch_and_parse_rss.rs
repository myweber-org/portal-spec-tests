use reqwest;
use rss::Channel;
use std::error::Error;

pub async fn fetch_and_parse_rss(feed_url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::get(feed_url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;
    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let url = "https://example.com/feed.rss";
    let channel = fetch_and_parse_rss(url).await?;

    println!("Feed Title: {}", channel.title());
    println!("Feed Description: {}", channel.description());
    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("Item Title: {}", title);
        }
    }
    Ok(())
}