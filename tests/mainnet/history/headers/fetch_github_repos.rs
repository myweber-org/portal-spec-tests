use reqwest;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let username = if args.len() > 1 {
        args[1].clone()
    } else {
        println!("Usage: {} <github_username>", args[0]);
        return Ok(());
    };

    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let repos: Value = response.json().await?;
        if let Some(repos_array) = repos.as_array() {
            println!("Repositories for {}:", username);
            for repo in repos_array {
                if let Some(name) = repo["name"].as_str() {
                    if let Some(description) = repo["description"].as_str() {
                        println!("- {}: {}", name, description);
                    } else {
                        println!("- {} (No description)", name);
                    }
                }
            }
        } else {
            println!("No repositories found or invalid response.");
        }
    } else {
        eprintln!("Failed to fetch repositories. Status: {}", response.status());
    }

    Ok(())
}