/// A fixed-size buffer for writing CSV data that avoids UTF-8 validation overhead
pub struct CsvBuffer {
    buffer: Box<[u8]>,
    position: usize,
}

impl CsvBuffer {
    /// Creates a new CsvBuffer with the specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: vec![0; capacity].into_boxed_slice(),
            position: 0,
        }
    }

    /// Writes a quoted string and returns the number of bytes written
    #[inline]
    pub fn write_quoted_str(&mut self, s: &str) -> usize {
        // Each write starts from the beginning of the internal temp buffer
        self.position = 0;
        // Fast path: we know the exact size needed
        let bytes = s.as_bytes();
        let total_len = bytes.len() + 3; // quotes + newline

        // Write the opening quote
        self.buffer[0] = b'"';

        // Copy the string bytes directly into 1..1+len (exclusive end)
        let start = 1usize;
        let end = start + bytes.len();
        self.buffer[start..end].copy_from_slice(bytes);

        // Write the closing quote and newline
        self.buffer[end] = b'"';
        self.buffer[end + 1] = b'\n';

        self.position = total_len;
        total_len
    }

    /// Writes a quoted pair of values (name, quantity) in CSV format
    #[inline]
    pub fn write_csv_pair(&mut self, name: &str, quantity: i32) -> usize {
        self.position = 0;
        let name_bytes = name.as_bytes();
        // Write first quoted field
        let mut pos = 0usize;
        self.buffer[pos] = b'"';
        pos += 1;

        let name_end = pos + name_bytes.len();
        self.buffer[pos..name_end].copy_from_slice(name_bytes);
        pos = name_end;

        // closing quote for name
        self.buffer[pos] = b'"';
        pos += 1;

        // separator
        self.buffer[pos] = b',';
        pos += 1;

        // opening quote for quantity
        self.buffer[pos] = b'"';
        pos += 1;

        // Write quantity using fast integer formatting into the remaining slice
        let qty_start = pos;
        let qty_len = write_int(quantity, &mut self.buffer[qty_start..]);
        let qty_end = qty_start + qty_len;

        // Write closing quote and newline
        self.buffer[qty_end] = b'"';
        self.buffer[qty_end + 1] = b'\n';

        self.position = qty_end + 2;
        self.position
    }

    /// Returns a slice of the buffer containing the written data
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buffer[..self.position]
    }

    /// Returns true if the buffer has enough capacity for n more bytes
    #[inline]
    pub fn can_fit(&self, n: usize) -> bool {
        // Buffer is used as a temporary per-line buffer that always writes from
        // the beginning; capacity check should therefore only ensure the
        // requested length fits in the backing buffer.
        n <= self.buffer.len()
    }
}

/// Fast integer to string conversion without going through fmt
#[inline]
fn write_int(n: i32, buf: &mut [u8]) -> usize {
    // Handle zero quickly
    if n == 0 {
        buf[0] = b'0';
        return 1;
    }

    // Use a wider integer type to avoid overflow for i32::MIN
    let mut pos = 0usize;
    let mut num = n as i64;
    if num < 0 {
        buf[0] = b'-';
        pos = 1;
        num = -num;
    }

    // Count digits
    let mut digits = 0usize;
    let mut temp = num;
    while temp > 0 {
        digits += 1;
        temp /= 10;
    }

    let end_pos = pos + digits;
    let mut curr_pos = end_pos;
    while num > 0 {
        curr_pos -= 1;
        buf[curr_pos] = b'0' + (num % 10) as u8;
        num /= 10;
    }

    end_pos
}

// Unit tests for the buffer formatting helpers
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_int_positive() {
        let mut buf = [0u8; 16];
        let len = write_int(12345, &mut buf);
        assert_eq!(len, 5);
        assert_eq!(&buf[..len], b"12345");
    }

    #[test]
    fn test_write_int_negative() {
        let mut buf = [0u8; 16];
        let len = write_int(-42, &mut buf);
        assert_eq!(len, 3);
        assert_eq!(&buf[..len], b"-42");
    }

    #[test]
    fn test_write_int_min_max() {
        let mut buf = [0u8; 32];
        let len_min = write_int(i32::MIN, &mut buf);
        let s_min = std::str::from_utf8(&buf[..len_min]).unwrap();
        assert_eq!(s_min, i32::MIN.to_string());

        let len_max = write_int(i32::MAX, &mut buf);
        let s_max = std::str::from_utf8(&buf[..len_max]).unwrap();
        assert_eq!(s_max, i32::MAX.to_string());
    }

    #[test]
    fn test_write_quoted_str() {
        let mut b = CsvBuffer::with_capacity(64);
        let len = b.write_quoted_str("Hello, world");
        let s = std::str::from_utf8(b.as_bytes()).unwrap();
        assert_eq!(s, "\"Hello, world\"\n");
        assert_eq!(len, s.len());
    }

    #[test]
    fn test_write_csv_pair() {
        let mut b = CsvBuffer::with_capacity(64);
        let len = b.write_csv_pair("Card Name", 3);
        let s = std::str::from_utf8(b.as_bytes()).unwrap();
        assert_eq!(s, "\"Card Name\",\"3\"\n");
        assert_eq!(len, s.len());
    }
}
