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
            println!("User Profile:");
            println!("Username: {}", user.login);
            println!("ID: {}", user.id);
            println!("Profile URL: {}", user.html_url);
            println!("Name: {:?}", user.name);
            println!("Public Repositories: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}use reqwest;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub location: Option<String>,
    pub public_repos: u32,
    pub followers: u32,
    pub following: u32,
}

pub async fn fetch_github_user(username: &str) -> Result<GitHubUser, Box<dyn Error>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_existing_user() {
        let result = fetch_github_user("octocat").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.login, "octocat");
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_fetch_nonexistent_user() {
        let result = fetch_github_user("nonexistentuser123456789").await;
        assert!(result.is_err());
    }
}