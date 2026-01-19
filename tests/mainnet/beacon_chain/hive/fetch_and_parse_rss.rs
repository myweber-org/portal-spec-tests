use clap::Parser;
use reqwest;
use rss::Channel;
use std::error::Error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let content = reqwest::get(&args.url).await?.bytes().await?;
    let channel = Channel::read_from(&content[..])?;

    println!("Feed: {}", channel.title());
    for item in channel.items() {
        if let Some(title) = item.title() {
            println!(" - {}", title);
        }
    }

    Ok(())
}