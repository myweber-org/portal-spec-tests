
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ValidationError {
    details: String,
}

impl ValidationError {
    fn new(msg: &str) -> ValidationError {
        ValidationError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for ValidationError {
    fn description(&self) -> &str {
        &self.details
    }
}

pub fn validate_numeric_data(data: &[f64]) -> Result<(), ValidationError> {
    if data.is_empty() {
        return Err(ValidationError::new("Data slice cannot be empty"));
    }

    for &value in data {
        if value.is_nan() || value.is_infinite() {
            return Err(ValidationError::new("Data contains invalid numeric values"));
        }
    }

    Ok(())
}

pub fn normalize_data(data: &[f64]) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }

    let min = data.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max = data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
    let range = max - min;

    if range == 0.0 {
        return vec![0.5; data.len()];
    }

    data.iter()
        .map(|&x| (x - min) / range)
        .collect()
}

pub fn calculate_statistics(data: &[f64]) -> (f64, f64, f64) {
    let count = data.len() as f64;
    let sum: f64 = data.iter().sum();
    let mean = sum / count;

    let variance: f64 = data
        .iter()
        .map(|&value| {
            let diff = mean - value;
            diff * diff
        })
        .sum::<f64>()
        / count;

    let std_dev = variance.sqrt();

    (mean, variance, std_dev)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_numeric_data() {
        let valid_data = vec![1.0, 2.0, 3.0];
        assert!(validate_numeric_data(&valid_data).is_ok());

        let invalid_data = vec![1.0, f64::NAN, 3.0];
        assert!(validate_numeric_data(&invalid_data).is_err());
    }

    #[test]
    fn test_normalize_data() {
        let data = vec![0.0, 5.0, 10.0];
        let normalized = normalize_data(&data);
        assert_eq!(normalized, vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn test_calculate_statistics() {
        let data = vec![2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0];
        let (mean, variance, std_dev) = calculate_statistics(&data);
        assert!((mean - 5.0).abs() < 0.0001);
        assert!((variance - 4.0).abs() < 0.0001);
        assert!((std_dev - 2.0).abs() < 0.0001);
    }
}