
use reqwest;
use rss::Channel;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <rss_feed_url>", args[0]);
        std::process::exit(1);
    }
    let url = &args[1];

    let content = reqwest::get(url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    println!("Feed: {}", channel.title());
    for item in channel.items() {
        let title = item.title().unwrap_or("No title");
        let link = item.link().unwrap_or("No link");
        println!("- {}: {}", title, link);
    }

    Ok(())
}use rss::Channel;
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
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let feed_url = "https://example.com/feed.rss";
    let channel = fetch_rss_feed(feed_url)?;
    print_feed_items(&channel);
    Ok(())
}