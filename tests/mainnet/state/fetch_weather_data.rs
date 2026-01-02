
use reqwest;
use serde_json::Value;
use std::error::Error;

pub async fn fetch_weather_data(api_key: &str, city: &str) -> Result<Value, Box<dyn Error>> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    
    if !response.status().is_success() {
        return Err(format!("API request failed with status: {}", response.status()).into());
    }
    
    let weather_data: Value = response.json().await?;
    
    if weather_data.get("cod").and_then(|c| c.as_str()) != Some("200") {
        return Err(format!("Weather API error: {:?}", weather_data.get("message")).into());
    }
    
    Ok(weather_data)
}

pub fn extract_temperature(weather_data: &Value) -> Option<f64> {
    weather_data
        .get("main")?
        .get("temp")?
        .as_f64()
}

pub fn extract_weather_description(weather_data: &Value) -> Option<String> {
    let weather_array = weather_data.get("weather")?.as_array()?;
    weather_array
        .first()?
        .get("description")?
        .as_str()
        .map(|s| s.to_string())
}