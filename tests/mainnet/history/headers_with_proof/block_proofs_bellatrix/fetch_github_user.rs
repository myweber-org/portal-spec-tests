
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_valid_user() {
        let result = fetch_github_user("octocat").await;
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.login, "octocat");
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_fetch_invalid_user() {
        let result = fetch_github_user("nonexistentuser123456789").await;
        assert!(result.is_err());
    }
}