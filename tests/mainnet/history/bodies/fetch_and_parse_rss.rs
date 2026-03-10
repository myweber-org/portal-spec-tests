use reqwest;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::BufReader;

#[derive(Debug)]
pub struct RssItem {
    pub title: String,
    pub link: String,
    pub description: String,
}

pub async fn fetch_rss_feed(url: &str) -> Result<Vec<RssItem>, Box<dyn std::error::Error>> {
    let response = reqwest::get(url).await?;
    let content = response.text().await?;
    
    let mut items = Vec::new();
    let mut reader = Reader::from_str(&content);
    reader.trim_text(true);
    
    let mut buf = Vec::new();
    let mut current_item: Option<RssItem> = None;
    let mut current_tag = String::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                
                if current_tag == "item" {
                    current_item = Some(RssItem {
                        title: String::new(),
                        link: String::new(),
                        description: String::new(),
                    });
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref mut item) = current_item {
                    let text = e.unescape().unwrap().to_string();
                    
                    match current_tag.as_str() {
                        "title" => item.title = text,
                        "link" => item.link = text,
                        "description" => item.description = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                
                if tag_name == "item" {
                    if let Some(item) = current_item.take() {
                        items.push(item);
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_fetch_rss() {
        let url = "https://example.com/feed.rss";
        let result = fetch_rss_feed(url).await;
        
        match result {
            Ok(items) => {
                println!("Fetched {} RSS items", items.len());
                for item in items.iter().take(3) {
                    println!("Title: {}", item.title);
                    println!("Link: {}", item.link);
                }
            }
            Err(e) => {
                println!("Failed to fetch RSS: {}", e);
            }
        }
    }
}