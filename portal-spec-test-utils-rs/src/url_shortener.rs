
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use lazy_static::lazy_static;
use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;

lazy_static! {
    static ref URL_REGEX: Regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
}

#[derive(Clone)]
pub struct UrlShortener {
    storage: Arc<Mutex<HashMap<String, String>>>,
    base_url: String,
}

impl UrlShortener {
    pub fn new(base_url: &str) -> Self {
        Self {
            storage: Arc::new(Mutex::new(HashMap::new())),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub fn shorten(&self, original_url: &str) -> Result<String, &'static str> {
        if !URL_REGEX.is_match(original_url) {
            return Err("Invalid URL format");
        }

        let short_code = self.generate_short_code();
        let short_url = format!("{}/{}", self.base_url, short_code);

        let mut storage = self.storage.lock().unwrap();
        storage.insert(short_code.clone(), original_url.to_string());

        Ok(short_url)
    }

    pub fn resolve(&self, short_code: &str) -> Option<String> {
        let storage = self.storage.lock().unwrap();
        storage.get(short_code).cloned()
    }

    fn generate_short_code(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect()
    }

    pub fn get_stats(&self) -> usize {
        let storage = self.storage.lock().unwrap();
        storage.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_shortener() {
        let shortener = UrlShortener::new("https://short.url");
        
        let original = "https://www.example.com/some/long/path";
        let short_url = shortener.shorten(original).unwrap();
        
        assert!(short_url.starts_with("https://short.url/"));
        assert_eq!(shortener.get_stats(), 1);
        
        let code = short_url.split('/').last().unwrap();
        let resolved = shortener.resolve(code).unwrap();
        assert_eq!(resolved, original);
    }

    #[test]
    fn test_invalid_url() {
        let shortener = UrlShortener::new("https://short.url");
        let result = shortener.shorten("not-a-valid-url");
        assert!(result.is_err());
    }
}