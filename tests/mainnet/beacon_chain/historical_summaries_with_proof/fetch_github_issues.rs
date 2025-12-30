use reqwest;
use serde::Deserialize;
use std::env;

#[derive(Deserialize, Debug)]
struct Issue {
    number: u64,
    title: String,
    html_url: String,
    state: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <owner> <repo>", args[0]);
        std::process::exit(1);
    }
    let owner = &args[1];
    let repo = &args[2];

    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);
    let response = client
        .get(&url)
        .header("User-Agent", "rust-reqwest")
        .send()
        .await?;

    if response.status().is_success() {
        let issues: Vec<Issue> = response.json().await?;
        let open_issues: Vec<&Issue> = issues.iter().filter(|i| i.state == "open").collect();

        println!("Open issues for {}/{}:", owner, repo);
        for issue in open_issues {
            println!("#{}: {}", issue.number, issue.title);
            println!("URL: {}", issue.html_url);
            println!("---");
        }
        println!("Total open issues: {}", open_issues.len());
    } else {
        eprintln!("Failed to fetch issues: {}", response.status());
    }

    Ok(())
}