use reqwest;
use serde_json::Value;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<f64, Box<dyn std::error::Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let json: Value = response.json().await?;
    
    match json["main"]["temp"].as_f64() {
        Some(temp) => Ok(temp),
        None => Err("Temperature data not found in response".into()),
    }
}