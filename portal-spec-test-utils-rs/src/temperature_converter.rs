
pub fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    (celsius * 9.0 / 5.0) + 32.0
}

pub fn celsius_to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

pub fn fahrenheit_to_celsius(fahrenheit: f64) -> f64 {
    (fahrenheit - 32.0) * 5.0 / 9.0
}

pub fn fahrenheit_to_kelvin(fahrenheit: f64) -> f64 {
    celsius_to_kelvin(fahrenheit_to_celsius(fahrenheit))
}

pub fn kelvin_to_celsius(kelvin: f64) -> f64 {
    kelvin - 273.15
}

pub fn kelvin_to_fahrenheit(kelvin: f64) -> f64 {
    celsius_to_fahrenheit(kelvin_to_celsius(kelvin))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_celsius_to_fahrenheit() {
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < f64::EPSILON);
        assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_celsius_to_kelvin() {
        assert!((celsius_to_kelvin(0.0) - 273.15).abs() < f64::EPSILON);
        assert!((celsius_to_kelvin(-273.15) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_fahrenheit_to_celsius() {
        assert!((fahrenheit_to_celsius(32.0) - 0.0).abs() < f64::EPSILON);
        assert!((fahrenheit_to_celsius(212.0) - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_kelvin_to_celsius() {
        assert!((kelvin_to_celsius(273.15) - 0.0).abs() < f64::EPSILON);
        assert!((kelvin_to_celsius(0.0) - (-273.15)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_full_conversion_cycle() {
        let original_celsius = 25.0;
        let fahrenheit = celsius_to_fahrenheit(original_celsius);
        let kelvin = celsius_to_kelvin(original_celsius);
        let back_to_celsius_from_f = fahrenheit_to_celsius(fahrenheit);
        let back_to_celsius_from_k = kelvin_to_celsius(kelvin);
        
        assert!((original_celsius - back_to_celsius_from_f).abs() < f64::EPSILON);
        assert!((original_celsius - back_to_celsius_from_k).abs() < f64::EPSILON);
    }
}