
use reqwest;
use serde_json::Value;
use std::error::Error;

#[derive(Debug)]
pub struct RepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub stars: u64,
    pub forks: u64,
    pub language: Option<String>,
}

pub async fn fetch_repo_info(owner: &str, repo: &str) -> Result<RepoInfo, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/{}", owner, repo);
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-GitHub-API-Client")
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }

    let json: Value = response.json().await?;

    let repo_info = RepoInfo {
        name: json["name"].as_str().unwrap_or("").to_string(),
        description: json["description"].as_str().map(|s| s.to_string()),
        stars: json["stargazers_count"].as_u64().unwrap_or(0),
        forks: json["forks_count"].as_u64().unwrap_or(0),
        language: json["language"].as_str().map(|s| s.to_string()),
    };

    Ok(repo_info)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito;

    #[tokio::test]
    async fn test_fetch_repo_info_success() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("GET", "/repos/rust-lang/rust")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "name": "rust",
                "description": "Empowering everyone to build reliable and efficient software.",
                "stargazers_count": 90000,
                "forks_count": 12000,
                "language": "Rust"
            }"#)
            .create_async()
            .await;

        let url = server.url();
        let result = fetch_repo_info_with_url(&url, "rust-lang", "rust").await;
        mock.assert_async().await;

        assert!(result.is_ok());
        let info = result.unwrap();
        assert_eq!(info.name, "rust");
        assert_eq!(info.stars, 90000);
        assert_eq!(info.language.unwrap(), "Rust");
    }

    async fn fetch_repo_info_with_url(base_url: &str, owner: &str, repo: &str) -> Result<RepoInfo, Box<dyn Error>> {
        let url = format!("{}/repos/{}/{}", base_url, owner, repo);
        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "Rust-GitHub-API-Client")
            .send()
            .await?;

        let json: Value = response.json().await?;
        let repo_info = RepoInfo {
            name: json["name"].as_str().unwrap_or("").to_string(),
            description: json["description"].as_str().map(|s| s.to_string()),
            stars: json["stargazers_count"].as_u64().unwrap_or(0),
            forks: json["forks_count"].as_u64().unwrap_or(0),
            language: json["language"].as_str().map(|s| s.to_string()),
        };
        Ok(repo_info)
    }
}