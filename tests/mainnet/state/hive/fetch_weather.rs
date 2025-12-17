use reqwest;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

#[derive(Debug, Deserialize)]
struct WeatherResponse {
    main: Main,
    name: String,
}

#[derive(Debug, Deserialize)]
struct Main {
    temp: f64,
    humidity: u8,
}

#[derive(Debug)]
struct CacheEntry {
    data: WeatherResponse,
    timestamp: SystemTime,
}

struct WeatherFetcher {
    client: reqwest::Client,
    cache: Arc<Mutex<HashMap<String, CacheEntry>>>,
    api_key: String,
}

impl WeatherFetcher {
    fn new(api_key: String) -> Self {
        WeatherFetcher {
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            api_key,
        }
    }

    async fn fetch_weather(&self, city: &str) -> Result<WeatherResponse, Box<dyn std::error::Error>> {
        let cache_key = city.to_lowercase();
        
        {
            let cache = self.cache.lock().unwrap();
            if let Some(entry) = cache.get(&cache_key) {
                if entry.timestamp.elapsed().unwrap_or(Duration::from_secs(0)) < Duration::from_secs(300) {
                    return Ok(WeatherResponse {
                        main: Main {
                            temp: entry.data.main.temp,
                            humidity: entry.data.main.humidity,
                        },
                        name: entry.data.name.clone(),
                    });
                }
            }
        }

        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            city, self.api_key
        );

        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(format!("API request failed with status: {}", response.status()).into());
        }

        let weather_data: WeatherResponse = response.json().await?;
        
        let mut cache = self.cache.lock().unwrap();
        cache.insert(
            cache_key,
            CacheEntry {
                data: WeatherResponse {
                    main: Main {
                        temp: weather_data.main.temp,
                        humidity: weather_data.main.humidity,
                    },
                    name: weather_data.name.clone(),
                },
                timestamp: SystemTime::now(),
            },
        );

        Ok(weather_data)
    }

    fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.lock().unwrap();
        let total_entries = cache.len();
        let valid_entries = cache.values()
            .filter(|entry| entry.timestamp.elapsed().unwrap_or(Duration::from_secs(0)) < Duration::from_secs(300))
            .count();
        (total_entries, valid_entries)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("WEATHER_API_KEY")
        .unwrap_or_else(|_| "demo_key".to_string());
    
    let fetcher = WeatherFetcher::new(api_key);
    
    match fetcher.fetch_weather("London").await {
        Ok(weather) => {
            println!("Weather in {}: {:.1}°C, {}% humidity", 
                    weather.name, weather.main.temp, weather.main.humidity);
        }
        Err(e) => {
            eprintln!("Failed to fetch weather: {}", e);
        }
    }
    
    let (total, valid) = fetcher.get_cache_stats();
    println!("Cache stats: {} total entries, {} valid entries", total, valid);
    
    Ok(())
}