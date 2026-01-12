use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize)]
struct RepoInfo {
    stargazers_count: u64,
}

pub async fn get_star_count(owner: &str, repo: &str) -> Result<u64, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-Stars-Fetcher")
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()).into());
    }
    
    let repo_info: RepoInfo = response.json().await?;
    Ok(repo_info.stargazers_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_stars() {
        let result = get_star_count("rust-lang", "rust").await;
        assert!(result.is_ok());
        let stars = result.unwrap();
        assert!(stars > 0);
    }
}