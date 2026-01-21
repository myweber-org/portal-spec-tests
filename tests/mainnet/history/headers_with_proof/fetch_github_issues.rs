use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::error::Error;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    owner: String,
    #[arg(short, long)]
    repo: String,
    #[arg(short, long, default_value_t = 10)]
    limit: u8,
}

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

fn fetch_issues(owner: &str, repo: &str, limit: u8) -> Result<Vec<Issue>, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
    let client = Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-cli-tool")
        .query(&[("per_page", limit), ("state", "all")])
        .send()?;

    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }

    let issues: Vec<Issue> = response.json()?;
    Ok(issues)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    let issues = fetch_issues(&args.owner, &args.repo, args.limit)?;

    println!("Recent issues for {}/{}:", args.owner, args.repo);
    for issue in issues {
        println!(
            "#{} [{}] {} - {}",
            issue.number, issue.state, issue.title, issue.html_url
        );
    }

    Ok(())
}