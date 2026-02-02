use rss::Channel;
use std::error::Error;

pub fn fetch_rss_feed(url: &str) -> Result<Channel, Box<dyn Error>> {
    let content = reqwest::blocking::get(url)?.text()?;
    let channel = Channel::read_from(content.as_bytes())?;
    Ok(channel)
}

pub fn print_feed_items(channel: &Channel) {
    println!("Feed Title: {}", channel.title());
    println!("Feed Link: {}", channel.link());
    println!("\nLatest Items:");
    for item in channel.items().iter().take(5) {
        if let Some(title) = item.title() {
            println!("- {}", title);
            if let Some(link) = item.link() {
                println!("  Link: {}", link);
            }
            println!();
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let feed_url = "https://example.com/feed.rss";
    let channel = fetch_rss_feed(feed_url)?;
    print_feed_items(&channel);
    Ok(())
}