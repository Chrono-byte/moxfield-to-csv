use std::cell::RefCell;
use std::thread_local;
use crate::models::{ProcessedDeck, error::Error};
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::io::Read;
use std::fs::File;
use bytecount;
use crate::buffer::CsvBuffer;

// Increased buffer size for better I/O performance (32KB)
const BUFFER_SIZE: usize = 32 * 1024;

// Thread-local buffer for CSV line formatting
thread_local! {
    static CSV_BUFFER: RefCell<CsvBuffer> = RefCell::new(CsvBuffer::with_capacity(256));
}

/// A struct that handles writing Magic: The Gathering deck data to CSV format
pub struct CsvWriter {
    // Controls whether to output in single-column (true) or two-column (false) format
    single_column: bool,
}

impl CsvWriter {
    /// Creates a new CsvWriter instance
    pub fn new(single_column: bool) -> Self {
        Self { single_column }
    }

    /// Writes the deck data to a CSV file
    pub fn write_to_file(&self, deck: &ProcessedDeck, path: &Path) -> Result<usize, Error> {
        let file = File::create(path)?;
        let mut writer = BufWriter::with_capacity(BUFFER_SIZE, file);
        let mut bytes_written = 0;

        if self.single_column {
            for (name, quantity) in &deck.main_deck {
                for _ in 0..*quantity {
                    bytes_written += write_quoted_line(&mut writer, name)?;
                }
            }
        } else {
            for (name, quantity) in &deck.main_deck {
                bytes_written += write_csv_line(&mut writer, name, *quantity)?;
            }
        }

        if !deck.commanders.is_empty() {
            bytes_written += writer.write(b"\n")?;

            for name in &deck.commanders {
                if self.single_column {
                    bytes_written += write_quoted_line(&mut writer, name)?;
                } else {
                    bytes_written += write_csv_line(&mut writer, name, 1)?;
                }
            }
        }

        writer.flush()?;
        Ok(bytes_written)
    }

    /// Generates a preview of a CSV file along with its line count
    pub fn preview_file(&self, path: &Path) -> Result<(String, usize), Error> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let file_size = metadata.len();

        if file_size > 2000 {
            let mut buffer = Vec::with_capacity(2000);
            let reader = io::BufReader::with_capacity(BUFFER_SIZE, file);
            io::copy(&mut reader.take(2000), &mut buffer)?;

            let preview = String::from_utf8_lossy(&buffer);
            let line_count = count_lines(path)?;

            Ok((preview.into_owned(), line_count))
        } else {
            let mut content = String::new();
            io::BufReader::new(file).read_to_string(&mut content)?;
            let line_count = content.lines().count();
            Ok((content, line_count))
        }
    }
}

/// Writes a single quoted line to the output
fn write_quoted_line(writer: &mut impl Write, text: &str) -> Result<usize, io::Error> {
    CSV_BUFFER.with(|buffer| -> Result<usize, io::Error> {
        let mut buffer = buffer.borrow_mut();
        let len = text.len() + 3; // quotes + newline
        if !buffer.can_fit(len) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "buffer capacity exceeded",
            ));
        }
        buffer.write_quoted_str(text);
        writer.write(buffer.as_bytes())
    })
}

/// Writes a CSV line with name and quantity
fn write_csv_line(writer: &mut impl Write, name: &str, quantity: i32) -> Result<usize, io::Error> {
    CSV_BUFFER.with(|buffer| -> Result<usize, io::Error> {
        let mut buffer = buffer.borrow_mut();
        let len = name.len() + 16; // quotes + comma + max int length + newline
        if !buffer.can_fit(len) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "buffer capacity exceeded",
            ));
        }
        buffer.write_csv_pair(name, quantity);
        writer.write(buffer.as_bytes())
    })
}

/// Counts the number of lines in a file efficiently using SIMD operations through the bytecount crate
fn count_lines(path: &Path) -> Result<usize, io::Error> {
    let file = File::open(path)?;
    let mut reader = io::BufReader::with_capacity(BUFFER_SIZE, file);
    let mut buffer = vec![0; BUFFER_SIZE];
    let mut count: usize = 0;
    
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        count += bytecount::count(&buffer[..bytes_read], b'\n');
    }

    Ok(count)
}
