use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct Repository {
    name: String,
    full_name: String,
    description: Option<String>,
    html_url: String,
    stargazers_count: u32,
    forks_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let username = "rust-lang";
    let url = format!("https://api.github.com/users/{}/repos", username);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        
        println!("Repositories for user '{}':", username);
        println!("{:-<60}", "");
        
        for repo in repos.iter().take(5) {
            println!("Name: {}", repo.name);
            println!("Full Name: {}", repo.full_name);
            if let Some(desc) = &repo.description {
                println!("Description: {}", desc);
            }
            println!("URL: {}", repo.html_url);
            println!("Stars: {}", repo.stargazers_count);
            println!("Forks: {}", repo.forks_count);
            println!("{:-<60}", "");
        }
        
        let total_stars: u32 = repos.iter().map(|r| r.stargazers_count).sum();
        println!("Total repositories: {}", repos.len());
        println!("Total stars across all repos: {}", total_stars);
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
    }

    Ok(())
}