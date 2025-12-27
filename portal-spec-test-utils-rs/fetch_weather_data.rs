
use reqwest;
use serde::Deserialize;
use std::error::Error;

#[derive(Deserialize, Debug)]
struct WeatherData {
    main: Main,
    name: String,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: u8,
}

pub async fn get_current_weather(api_key: &str, city: &str) -> Result<WeatherData, Box<dyn Error>> {
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
    use mockito;

    #[tokio::test]
    async fn test_get_current_weather_success() {
        let mut server = mockito::Server::new();
        let mock = server
            .mock("GET", "/data/2.5/weather")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"name":"London","main":{"temp":15.5,"feels_like":14.2,"humidity":65}}"#)
            .create();

        let api_key = "test_key";
        let city = "London";
        let url = server.url();

        let result = get_current_weather_with_url(api_key, city, &url).await;
        mock.assert();

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data.name, "London");
        assert_eq!(data.main.temp, 15.5);
    }

    async fn get_current_weather_with_url(api_key: &str, city: &str, base_url: &str) -> Result<WeatherData, Box<dyn Error>> {
        let url = format!(
            "{}/data/2.5/weather?q={}&appid={}&units=metric",
            base_url, city, api_key
        );

        let response = reqwest::get(&url).await?;
        let weather_data: WeatherData = response.json().await?;

        Ok(weather_data)
    }
}