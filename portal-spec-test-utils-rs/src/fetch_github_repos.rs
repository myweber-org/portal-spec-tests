use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    description: Option<String>,
    stargazers_count: u32,
    html_url: String,
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
    let username = "rust-lang";
    println!("Fetching repositories for user: {}", username);
    
    let repos = fetch_repositories(username).await?;
    
    println!("Found {} repositories:", repos.len());
    for repo in repos.iter().take(5) {
        println!("- {} ({} stars)", repo.name, repo.stargazers_count);
        if let Some(desc) = &repo.description {
            println!("  Description: {}", desc);
        }
        println!("  URL: {}", repo.html_url);
        println!();
    }
    
    Ok(())
}