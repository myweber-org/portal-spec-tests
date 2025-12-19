use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    state: String,
    html_url: String,
}

async fn fetch_open_issues(owner: &str, repo: &str) -> Result<Vec<Issue>, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-API-Client")
        .query(&[("state", "open")])
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        Ok(issues)
    } else {
        Err(format!("Failed to fetch issues: {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let owner = "rust-lang";
    let repo = "rust";

    match fetch_open_issues(owner, repo).await {
        Ok(issues) => {
            println!("Open issues for {}/{}:", owner, repo);
            for issue in issues {
                println!("#{} - {} ({})", issue.number, issue.title, issue.html_url);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}