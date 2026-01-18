use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct CsvMerger {
    delimiter: char,
    skip_header: bool,
}

impl CsvMerger {
    pub fn new(delimiter: char, skip_header: bool) -> Self {
        CsvMerger {
            delimiter,
            skip_header,
        }
    }

    pub fn merge_files<P: AsRef<Path>>(
        &self,
        input_paths: &[P],
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        let mut output_file = File::create(output_path)?;
        let mut first_file = true;

        for path in input_paths {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            if self.skip_header && !first_file {
                lines.next();
            }

            for line_result in lines {
                let line = line_result?;
                writeln!(output_file, "{}", line)?;
            }

            first_file = false;
        }

        Ok(())
    }

    pub fn merge_with_custom_processing<P: AsRef<Path>, F>(
        &self,
        input_paths: &[P],
        output_path: P,
        mut processor: F,
    ) -> Result<(), Box<dyn Error>>
    where
        F: FnMut(&str) -> Option<String>,
    {
        let mut output_file = File::create(output_path)?;
        let mut first_file = true;

        for path in input_paths {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            let mut lines = reader.lines();

            if self.skip_header && !first_file {
                lines.next();
            }

            for line_result in lines {
                let original_line = line_result?;
                if let Some(processed_line) = processor(&original_line) {
                    writeln!(output_file, "{}", processed_line)?;
                }
            }

            first_file = false;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use tempfile::NamedTempFile;

    #[test]
    fn test_basic_merge() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "a,b,c\n1,2,3\n4,5,6").unwrap();
        std::fs::write(&file2, "a,b,c\n7,8,9\n10,11,12").unwrap();

        let merger = CsvMerger::new(',', true);
        merger
            .merge_files(&[&file1, &file2], &output_file)
            .unwrap();

        let mut content = String::new();
        std::fs::File::open(&output_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "a,b,c\n1,2,3\n4,5,6\n7,8,9\n10,11,12\n");
    }

    #[test]
    fn test_keep_all_headers() {
        let file1 = NamedTempFile::new().unwrap();
        let file2 = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        std::fs::write(&file1, "header1,header2\nvalue1,value2").unwrap();
        std::fs::write(&file2, "header1,header2\nvalue3,value4").unwrap();

        let merger = CsvMerger::new(',', false);
        merger
            .merge_files(&[&file1, &file2], &output_file)
            .unwrap();

        let mut content = String::new();
        std::fs::File::open(&output_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();

        assert_eq!(content, "header1,header2\nvalue1,value2\nheader1,header2\nvalue3,value4\n");
    }
}