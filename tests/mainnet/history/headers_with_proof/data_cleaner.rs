use csv::{ReaderBuilder, WriterBuilder};
use std::error::Error;
use std::io;

pub fn clean_csv_data<R: io::Read, W: io::Write>(
    input: R,
    output: W,
    delimiter: u8,
    trim_fields: bool,
) -> Result<(), Box<dyn Error>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(input);

    let mut writer = WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(output);

    let headers = reader.headers()?.clone();
    writer.write_record(&headers)?;

    for result in reader.records() {
        let record = result?;
        let cleaned_record: Vec<String> = if trim_fields {
            record.iter().map(|field| field.trim().to_string()).collect()
        } else {
            record.iter().map(|field| field.to_string()).collect()
        };
        writer.write_record(&cleaned_record)?;
    }

    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_clean_csv_with_trimming() {
        let input_data = "name , age, city\nJohn Doe , 25, New York\n";
        let expected_output = "name,age,city\nJohn Doe,25,New York\n";

        let input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        clean_csv_data(input, &mut output, b',', true).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_clean_csv_without_trimming() {
        let input_data = "name , age, city\nJohn Doe , 25, New York\n";
        let expected_output = "name , age, city\nJohn Doe , 25, New York\n";

        let input = Cursor::new(input_data);
        let mut output = Cursor::new(Vec::new());

        clean_csv_data(input, &mut output, b',', false).unwrap();

        let result = String::from_utf8(output.into_inner()).unwrap();
        assert_eq!(result, expected_output);
    }
}