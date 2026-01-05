use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
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

async fn fetch_github_user(username: &str) -> Result<GitHubUser, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest-example")
        .send()
        .await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(format!("Failed to fetch user: HTTP {}", response.status()).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let username = "octocat";
    match fetch_github_user(username).await {
        Ok(user) => {
            println!("Successfully fetched user: {}", user.login);
            println!("ID: {}", user.id);
            println!("Profile: {}", user.html_url);
            if let Some(name) = user.name {
                println!("Name: {}", name);
            }
            println!("Public Repos: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}