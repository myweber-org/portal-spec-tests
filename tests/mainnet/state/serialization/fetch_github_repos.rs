use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("Usage: {} <github_username>", args[0]);
        return Ok(());
    };

    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/users/{}/repos", username);

    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-API-Client")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Vec<Repository> = response.json().await?;
        
        println!("GitHub repositories for user '{}':", username);
        println!("==========================================");
        
        for (i, repo) in repos.iter().enumerate() {
            println!("{}. {}", i + 1, repo.name);
            println!("   Full Name: {}", repo.full_name);
            println!("   URL: {}", repo.html_url);
            
            if let Some(desc) = &repo.description {
                println!("   Description: {}", desc);
            }
            
            println!("   Stars: {}", repo.stargazers_count);
            println!("   Forks: {}", repo.forks_count);
            
            if let Some(lang) = &repo.language {
                println!("   Language: {}", lang);
            }
            
            println!();
        }
        
        let total_stars: u32 = repos.iter().map(|r| r.stargazers_count).sum();
        let total_forks: u32 = repos.iter().map(|r| r.forks_count).sum();
        
        println!("Summary:");
        println!("  Total repositories: {}", repos.len());
        println!("  Total stars: {}", total_stars);
        println!("  Total forks: {}", total_forks);
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
        eprintln!("Make sure the username '{}' exists on GitHub.", username);
    }

    Ok(())
}