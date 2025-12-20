use clap::Parser;
use reqwest;
use rss::Channel;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The URL of the RSS feed to fetch and parse.
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let content = reqwest::get(&args.url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    println!("Feed: {}", channel.title());
    for item in channel.items() {
        let title = item.title().unwrap_or("No title");
        let link = item.link().unwrap_or("No link");
        println!("- {}: {}", title, link);
    }

    Ok(())
}