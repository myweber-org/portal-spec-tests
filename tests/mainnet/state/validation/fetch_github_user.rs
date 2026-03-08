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
use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    pub html_url: String,
    pub name: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    pub public_repos: u32,
}

pub async fn fetch_github_user(username: &str) -> Result<GitHubUser, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let user: GitHubUser = response.json().await?;
        Ok(user)
    } else {
        Err(response.error_for_status().unwrap_err())
    }
}use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
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

async fn fetch_github_user(username: &str) -> Result<GitHubUser, reqwest::Error> {
    let url = format!("https://api.github.com/users/{}", username);
    let client = reqwest::Client::new();
    
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-API-Client")
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
            println!("Name: {:?}", user.name);
            println!("Company: {:?}", user.company);
            println!("Location: {:?}", user.location);
            println!("Public Repos: {}", user.public_repos);
            println!("Followers: {}", user.followers);
            println!("Following: {}", user.following);
            println!("Profile URL: {}", user.html_url);
        }
        Err(e) => eprintln!("Error fetching user data: {}", e),
    }
}