
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
            base_url: base_url.to_string(),
        }
    }

    pub fn shorten(&self, original_url: &str) -> Result<String, &'static str> {
        if !URL_REGEX.is_match(original_url) {
            return Err("Invalid URL format");
        }

        let id = self.generate_id();
        let short_url = format!("{}/{}", self.base_url, id);

        let mut storage = self.storage.lock().unwrap();
        storage.insert(id.clone(), original_url.to_string());

        Ok(short_url)
    }

    pub fn resolve(&self, short_id: &str) -> Option<String> {
        let storage = self.storage.lock().unwrap();
        storage.get(short_id).cloned()
    }

    pub fn list_all(&self) -> Vec<(String, String)> {
        let storage = self.storage.lock().unwrap();
        storage
            .iter()
            .map(|(k, v)| (format!("{}/{}", self.base_url, k), v.clone()))
            .collect()
    }

    fn generate_id(&self) -> String {
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect::<String>()
            .to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_validation() {
        let shortener = UrlShortener::new("https://short.url");
        
        assert!(shortener.shorten("https://example.com").is_ok());
        assert!(shortener.shorten("http://example.com/path").is_ok());
        assert!(shortener.shorten("invalid-url").is_err());
        assert!(shortener.shorten("ftp://example.com").is_err());
    }

    #[test]
    fn test_shorten_and_resolve() {
        let shortener = UrlShortener::new("https://short.url");
        let original = "https://rust-lang.org";
        
        let short_url = shortener.shorten(original).unwrap();
        let id = short_url.split('/').last().unwrap();
        
        assert_eq!(shortener.resolve(id), Some(original.to_string()));
        assert_eq!(shortener.resolve("nonexistent"), None);
    }

    #[test]
    fn test_id_generation() {
        let shortener = UrlShortener::new("https://short.url");
        let id1 = shortener.generate_id();
        let id2 = shortener.generate_id();
        
        assert_eq!(id1.len(), 6);
        assert_ne!(id1, id2);
    }
}