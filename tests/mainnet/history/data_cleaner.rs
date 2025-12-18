use std::collections::HashSet;
use std::io::{self, BufRead, Write};

pub fn clean_data<R: BufRead, W: Write>(input: R, output: &mut W) -> io::Result<()> {
    let mut unique_lines = HashSet::new();
    
    for line in input.lines() {
        let line = line?;
        unique_lines.insert(line);
    }
    
    let mut sorted_lines: Vec<String> = unique_lines.into_iter().collect();
    sorted_lines.sort();
    
    for line in sorted_lines {
        writeln!(output, "{}", line)?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_clean_data() {
        let input_data = "banana\napple\nbanana\ncherry\napple\n";
        let expected_output = "apple\nbanana\ncherry\n";
        
        let input = Cursor::new(input_data);
        let mut output = Vec::new();
        
        clean_data(input, &mut output).unwrap();
        
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, expected_output);
    }
}