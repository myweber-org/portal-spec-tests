
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct Repository {
    stargazers_count: u32,
}

pub async fn get_star_count(owner: &str, repo: &str) -> Result<u32, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Stars-Fetcher")
        .send()
        .await?;

    if response.status().is_success() {
        let repo_info: Repository = response.json().await?;
        Ok(repo_info.stargazers_count)
    } else {
        Err(format!("Failed to fetch repository data: {}", response.status()).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_star_count() {
        let result = get_star_count("rust-lang", "rust").await;
        assert!(result.is_ok());
        let stars = result.unwrap();
        assert!(stars > 0);
    }
}