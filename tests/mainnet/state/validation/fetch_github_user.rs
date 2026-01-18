use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    public_repos: u32,
}

async fn fetch_github_user(username: &str) -> Result<GitHubUser, Box<dyn Error>> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Client")
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
            println!("User: {}", user.login);
            println!("ID: {}", user.id);
            println!("Profile: {}", user.html_url);
            println!("Public Repos: {}", user.public_repos);
            println!("Avatar: {}", user.avatar_url);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}