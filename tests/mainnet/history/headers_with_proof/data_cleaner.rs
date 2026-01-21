fn filter_empty_strings(strings: Vec<String>) -> Vec<String> {
    strings.into_iter().filter(|s| !s.trim().is_empty()).collect()
}