# Moxfield to CSV - AI Coding Guidelines

## Project Overview
This Rust CLI tool converts Magic: The Gathering decks from Moxfield.com to CSV format. It fetches deck data via REST API and outputs in two formats: standard CSV with quantities or one-card-per-line format.

## Architecture Patterns

### Module Structure
- **`main.rs`**: CLI orchestration using clap, coordinates data flow from input → API → processing → output
- **`client.rs`**: HTTP client for Moxfield API, handles URL parsing and deck ID extraction
- **`models.rs`**: Data structures for API responses (`MoxfieldResponse`, `ProcessedDeck`) and custom error types
- **`writer.rs`**: CSV output logic with performance optimizations
- **`buffer.rs`**: Custom byte buffer for fast CSV formatting without UTF-8 validation overhead

### Data Flow
```
URL/ID → extract_deck_id() → fetch_deck() → JSON parse → combine duplicates → sort → CSV write
```

## Performance Patterns

### Custom Buffer System
Use thread-local `CsvBuffer` for CSV formatting to avoid UTF-8 validation:
```rust
thread_local! {
    static CSV_BUFFER: RefCell<CsvBuffer> = RefCell::new(CsvBuffer::with_capacity(256));
}
```

### Fast Integer Formatting
Implement custom `write_int()` for i32 → string conversion without `fmt` overhead.

### SIMD Operations
Use `bytecount` crate for fast line counting with SIMD acceleration.

### Large I/O Buffers
Use 32KB buffers for file operations to minimize syscalls.

## Error Handling

### Custom Error Enum
```rust
pub enum Error {
    Network(reqwest::Error),  // HTTP/client errors
    Parse(String),           // URL/data parsing errors
    Io(std::io::Error),      // File system errors
    Other(String),           // Generic errors
}
```
Always implement `From<T>` conversions for external error types.

## Key Conventions

### API Interaction
- Base URL: `https://api.moxfield.com/v2/decks/all/{deck_id}`
- Include `User-Agent` header identifying the application
- Accept `application/json` responses

### URL Parsing
Support both full Moxfield URLs and raw deck IDs. Valid deck IDs contain only alphanumeric characters, hyphens, and underscores.

### Commander Validation
Enforce maximum of 2 commanders per deck (EDH format constraint).

### Card Processing
- Combine duplicate cards by summing quantities
- Sort main deck cards alphabetically by name
- Commanders are listed separately (not in main deck)

### Output Formats
- **Standard**: `"Card Name","Quantity"` (one row per unique card)
- **Single-column**: `"Card Name"` (one row per card instance, no quantities)

## Development Workflow

### Building
```bash
cargo build --release  # Optimized binary
cargo build           # Debug build
```

### Testing
```bash
cargo test           # Run unit tests
cargo test --doc     # Run doc tests
```

### Running
```bash
# Interactive mode
./target/release/moxfield-to-csv

# Direct URL with file output
./target/release/moxfield-to-csv "https://moxfield.com/decks/deck-id" -o deck.csv

# Single-column format
./target/release/moxfield-to-csv "deck-id" --single-column -o deck.csv
```

## Code Style Notes

### Async Runtime
Use `#[tokio::main]` for the main function when making HTTP requests.

### Dependency Management
- Use specific version requirements in `Cargo.toml`
- Enable features explicitly (e.g., `clap = { version = "4.0", features = ["derive"] }`)

### Memory Management
- Use `Box<[u8]>` for fixed-size byte buffers
- Prefer stack allocation for small, fixed-size buffers
- Use `RefCell` for interior mutability in thread-local storage

### String Handling
- Use `&str` for string slices, `String` for owned strings
- Convert between them explicitly with `.to_string()` or `.as_str()`

## Common Patterns

### HTTP Client Setup
```rust
let client = Client::builder()
    .build()
    .map_err(Error::Network)?;
```

### File Operations
```rust
let file = File::create(path)?;
let mut writer = BufWriter::with_capacity(BUFFER_SIZE, file);
```

### Result Propagation
Use `?` operator extensively for error handling, converting external errors to custom `Error` types.