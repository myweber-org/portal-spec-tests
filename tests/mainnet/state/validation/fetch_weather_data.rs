
use reqwest;
use serde_json::Value;
use std::time::Duration;

const API_URL: &str = "https://api.openweathermap.org/data/2.5/weather";
const MAX_RETRIES: u8 = 3;
const RETRY_DELAY_MS: u64 = 1000;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut last_error = None;

    for attempt in 1..=MAX_RETRIES {
        let params = [
            ("q", city),
            ("appid", api_key),
            ("units", "metric")
        ];

        match client.get(API_URL)
            .query(&params)
            .timeout(Duration::from_secs(10))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    return response.json::<Value>().await
                        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>);
                } else {
                    last_error = Some(format!("HTTP error: {}", response.status()));
                }
            }
            Err(e) => {
                last_error = Some(format!("Request failed: {}", e));
            }
        }

        if attempt < MAX_RETRIES {
            tokio::time::sleep(Duration::from_millis(RETRY_DELAY_MS)).await;
        }
    }

    Err(format!("Failed after {} attempts. Last error: {}", 
                MAX_RETRIES, 
                last_error.unwrap_or_else(|| "Unknown error".to_string())
    ).into())
}