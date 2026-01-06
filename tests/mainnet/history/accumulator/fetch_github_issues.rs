use clap::Parser;
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// GitHub repository owner
    #[arg(short, long)]
    owner: String,

    /// GitHub repository name
    #[arg(short, long)]
    repo: String,

    /// Maximum number of issues to display
    #[arg(short, long, default_value_t = 10)]
    limit: u32,
}

#[derive(Deserialize, Debug)]
struct Issue {
    number: u32,
    title: String,
    html_url: String,
    state: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let url = format!(
        "https://api.github.com/repos/{}/{}/issues",
        args.owner, args.repo
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        let open_issues: Vec<&Issue> = issues
            .iter()
            .filter(|issue| issue.state == "open")
            .take(args.limit as usize)
            .collect();

        println!("Open issues for {}/{}:", args.owner, args.repo);
        for issue in open_issues {
            println!("#{}: {}", issue.number, issue.title);
            println!("  URL: {}", issue.html_url);
            println!();
        }
    } else {
        eprintln!("Failed to fetch issues. Status: {}", response.status());
    }

    Ok(())
}