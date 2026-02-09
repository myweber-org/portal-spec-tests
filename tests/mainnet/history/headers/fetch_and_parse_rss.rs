
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
}