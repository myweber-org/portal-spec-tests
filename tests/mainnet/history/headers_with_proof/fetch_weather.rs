use std::error::Error;

const MOCK_API_URL: &str = "https://jsonplaceholder.typicode.com/posts/1";

#[derive(Debug, serde::Deserialize)]
struct WeatherData {
    title: String,
    body: String,
}

async fn fetch_weather(city: &str) -> Result<WeatherData, Box<dyn Error>> {
    let url = format!("{}?city={}", MOCK_API_URL, city);
    let response = reqwest::get(&url).await?;
    let weather: WeatherData = response.json().await?;
    Ok(weather)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <city_name>", args[0]);
        std::process::exit(1);
    }
    let city = &args[1];
    match fetch_weather(city).await {
        Ok(data) => println!("Weather in {}: {} - {}", city, data.title, data.body),
        Err(e) => eprintln!("Failed to fetch weather: {}", e),
    }
    Ok(())
}