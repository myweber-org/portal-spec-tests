
use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    name: Option<String>,
    company: Option<String>,
    blog: Option<String>,
    location: Option<String>,
    public_repos: u32,
    followers: u32,
    following: u32,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <github_username>", args[0]);
        std::process::exit(1);
    }

    let username = &args[1];
    let url = format!("https://api.github.com/users/{}", username);

    let client = reqwest::Client::builder()
        .user_agent("rust-github-api-client")
        .build()?;

    let response = client.get(&url).send().await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        println!("{:#?}", user);
    } else {
        eprintln!("Failed to fetch user '{}': {}", username, response.status());
        std::process::exit(1);
    }

    Ok(())
}