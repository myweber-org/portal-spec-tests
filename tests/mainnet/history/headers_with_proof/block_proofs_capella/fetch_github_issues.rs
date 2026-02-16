use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <owner> <repo>", args[0]);
        std::process::exit(1);
    }

    let owner = &args[1];
    let repo = &args[2];
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Issues-Fetcher")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        println!("Recent issues for {}/{}:", owner, repo);
        for issue in issues.iter().take(5) {
            println!("#{} [{}] {}", issue.number, issue.state, issue.title);
            println!("   {}", issue.html_url);
        }
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}