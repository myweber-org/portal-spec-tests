
use reqwest;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    weather: Vec<Weather>,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    humidity: u8,
    pressure: u16,
}

#[derive(Deserialize, Debug)]
struct Weather {
    description: String,
    icon: String,
}

pub async fn get_weather(api_key: &str, city: &str) -> Result<WeatherData, reqwest::Error> {
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        city, api_key
    );
    
    let response = reqwest::get(&url).await?;
    let weather_data: WeatherData = response.json().await?;
    
    Ok(weather_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_weather_fetch() {
        let api_key = "test_key";
        let city = "London";
        
        let result = get_weather(api_key, city).await;
        assert!(result.is_err());
    }
}