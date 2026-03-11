use reqwest;
use serde_json::Value;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let repo = "rust-lang/rust";
    let url = format!("https://api.github.com/repos/{}", repo);
    
    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Rust-Stars-Checker")
        .send()
        .await?;
    
    if response.status().is_success() {
        let json: Value = response.json().await?;
        let stars = json["stargazers_count"].as_u64().unwrap_or(0);
        println!("{} has {} stars", repo, stars);
    } else {
        eprintln!("Failed to fetch data: {}", response.status());
    }
    
    Ok(())
}