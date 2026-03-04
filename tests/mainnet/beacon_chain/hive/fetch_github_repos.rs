use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
    language: Option<String>,
}

async fn fetch_repositories(username: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        Ok(repos)
    } else {
        Err(format!("Failed to fetch repositories: {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }

    let username = &args[1];
    match fetch_repositories(username).await {
        Ok(repos) => {
            println!("Repositories for {}:", username);
            for repo in repos {
                println!("- {} ({})", repo.name, repo.html_url);
                if let Some(desc) = repo.description {
                    println!("  Description: {}", desc);
                }
                println!("  Stars: {}", repo.stargazers_count);
                if let Some(lang) = repo.language {
                    println!("  Language: {}", lang);
                }
                println!();
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}