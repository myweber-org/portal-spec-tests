use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    language: Option<String>,
}

async fn fetch_repositories(username: &str) -> Result<Vec<Repository>, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        Ok(repos)
    } else {
        Err(format!("Failed to fetch repositories: {}", response.status()).into())
    }
}

fn display_repositories(repos: &[Repository]) {
    for repo in repos {
        println!("Name: {}", repo.name);
        println!("Full Name: {}", repo.full_name);
        println!("URL: {}", repo.html_url);
        if let Some(desc) = &repo.description {
            println!("Description: {}", desc);
        }
        println!("Stars: {}", repo.stargazers_count);
        println!("Forks: {}", repo.forks_count);
        if let Some(lang) = &repo.language {
            println!("Language: {}", lang);
        }
        println!("---");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let username = "rust-lang";
    println!("Fetching repositories for user: {}", username);
    
    let repos = fetch_repositories(username).await?;
    println!("Found {} repositories:\n", repos.len());
    
    display_repositories(&repos);
    Ok(())
}