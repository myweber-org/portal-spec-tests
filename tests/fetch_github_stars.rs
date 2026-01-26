use reqwest;
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <owner>/<repo>", args[0]);
        std::process::exit(1);
    }

    let repo_path = &args[1];
    let url = format!("https://api.github.com/repos/{}", repo_path);

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "rust-github-stars-checker")
        .send()
        .await?;

    if response.status().is_success() {
        let json: Value = response.json().await?;
        if let Some(stars) = json.get("stargazers_count").and_then(Value::as_u64) {
            println!("{}: {} stars", repo_path, stars);
        } else {
            eprintln!("Failed to parse star count from response");
            std::process::exit(1);
        }
    } else {
        eprintln!("Failed to fetch repository: {}", response.status());
        std::process::exit(1);
    }

    Ok(())
}