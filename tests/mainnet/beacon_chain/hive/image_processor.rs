
use image::{DynamicImage, ImageFormat};
use std::fs;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn resize_image(input_path: &str, output_path: &str, width: u32, height: u32) -> Result<(), String> {
        let img = image::open(input_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        
        let resized = img.resize_exact(width, height, image::imageops::FilterType::Lanczos3);
        
        resized.save(output_path)
            .map_err(|e| format!("Failed to save resized image: {}", e))?;
        
        Ok(())
    }

    pub fn convert_format(input_path: &str, output_path: &str, target_format: ImageFormat) -> Result<(), String> {
        let img = image::open(input_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        
        let mut output_file = fs::File::create(output_path)
            .map_err(|e| format!("Failed to create output file: {}", e))?;
        
        img.write_to(&mut output_file, target_format)
            .map_err(|e| format!("Failed to write image in new format: {}", e))?;
        
        Ok(())
    }

    pub fn extract_metadata(input_path: &str) -> Result<(u32, u32, String), String> {
        let img = image::open(input_path)
            .map_err(|e| format!("Failed to open image: {}", e))?;
        
        let (width, height) = img.dimensions();
        let color_type = match img {
            DynamicImage::ImageLuma8(_) => "grayscale",
            DynamicImage::ImageLumaA8(_) => "grayscale with alpha",
            DynamicImage::ImageRgb8(_) => "RGB",
            DynamicImage::ImageRgba8(_) => "RGBA",
            _ => "unknown",
        }.to_string();
        
        Ok((width, height, color_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resize_image() {
        let input = "test_data/input.jpg";
        let output_file = NamedTempFile::new().unwrap();
        let output_path = output_file.path().to_str().unwrap();
        
        let result = ImageProcessor::resize_image(input, output_path, 100, 100);
        assert!(result.is_ok());
        
        let metadata = fs::metadata(output_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_extract_metadata() {
        let input = "test_data/input.jpg";
        let result = ImageProcessor::extract_metadata(input);
        
        assert!(result.is_ok());
        let (width, height, color_type) = result.unwrap();
        assert!(width > 0);
        assert!(height > 0);
        assert!(!color_type.is_empty());
    }
}