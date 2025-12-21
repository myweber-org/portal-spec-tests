use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct GitHubUser {
    login: String,
    id: u64,
    avatar_url: String,
    html_url: String,
    public_repos: u32,
    followers: u32,
    following: u32,
}

async fn fetch_github_user(username: &str) -> Result<GitHubUser, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Client")
        .send()
        .await?;
    
    response.json::<GitHubUser>().await
}

#[tokio::main]
async fn main() {
    match fetch_github_user("octocat").await {
        Ok(user) => {
            println!("User Profile:");
            println!("Username: {}", user.login);
            println!("ID: {}", user.id);
            println!("Profile URL: {}", user.html_url);
            println!("Public Repos: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
        }
        Err(e) => eprintln!("Failed to fetch user: {}", e),
    }
}