use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Repository {
    name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }
    let username = &args[1];

    let url = format!("https://api.github.com/users/{}/repos", username);
    let client = reqwest::Client::new();
    let repos: Vec<Repository> = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?
        .json()
        .await?;

    println!("Repositories for {}:", username);
    for repo in repos {
        println!("- {} ({})", repo.name, repo.html_url);
        if let Some(desc) = repo.description {
            println!("  Description: {}", desc);
        }
        println!("  Stars: {}", repo.stargazers_count);
        println!();
    }

    Ok(())
}