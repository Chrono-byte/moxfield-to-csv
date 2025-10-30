// Import the command-line argument parser
use clap::Parser;
// Import PathBuf for handling file paths
use std::path::PathBuf;

// Declare our module structure
mod buffer; // Module for efficient byte buffer
mod client; // Module for Moxfield API interaction
mod models; // Module for data structures and error handling
mod writer; // Module for CSV file writing

// Import the specific types we need
use crate::models::error::Error;
use client::MoxfieldClient;
use writer::CsvWriter;

/// Command-line interface definition using clap
#[derive(Parser)]
#[command(version, about = "Convert Moxfield decks to CSV format")]
struct Cli {
    /// The Moxfield deck URL or ID
    /// This can be either a full URL to a Moxfield deck or just the deck ID
    #[arg(value_name = "URL")]
    url: Option<String>,

    /// Path to the output file
    /// If not specified, the program will print the output to the console
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// When set, outputs one card per line without quantity column
    /// Useful for importing into systems that don't support quantities
    #[arg(short, long)]
    single_column: bool,
}

/// Entry point of the application
///
/// Uses tokio for async runtime as we need to make HTTP requests
#[tokio::main]
async fn main() -> Result<(), Error> {
    // Parse command line arguments using clap
    let cli = Cli::parse();

    // Get the deck URL either from command line arguments or by prompting the user
    let (deck_url, single_column) = if let Some(url) = cli.url {
        (url, cli.single_column)
    } else {
        println!("Enter Moxfield deck URL:");
        let mut input = String::new();
        // Read user input, converting any IO errors using our error type
        std::io::stdin().read_line(&mut input).map_err(Error::Io)?;
        let url = input.trim().to_string();

        println!("\nOutput one card per line without quantity column? [y/N]:");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(Error::Io)?;
        let single_column = input.trim().to_lowercase() == "y";

        (url, single_column)
    };

    // Extract the deck ID from the URL or validate the raw ID
    let deck_id = MoxfieldClient::extract_deck_id(&deck_url)
        .ok_or_else(|| Error::Parse("Invalid Moxfield URL format".to_string()))?;

    // Create a client and fetch the deck data from Moxfield
    // Initialize the Moxfield client (propagate any build errors)
    let client = MoxfieldClient::new()?;
    // Fetch the deck data asynchronously
    let deck = client.fetch_deck(&deck_id).await?;

    // Create a CSV writer with the specified format option
    let writer = CsvWriter::new(single_column);

    if let Some(output_path) = cli.output {
        // User specified an output file - write directly to it
        println!("Writing to file: {}", output_path.display());
        let bytes_written = writer.write_to_file(&deck, &output_path)?;
        println!(
            "\nDeck successfully written to {} ({} bytes)",
            output_path.display(),
            bytes_written
        );
    } else {
        // No output file specified - create a temporary file
        let temp_path =
            std::env::temp_dir().join(format!("deck-{}.csv", chrono::Utc::now().timestamp()));
        writer.write_to_file(&deck, &temp_path)?;

        // Display the CSV content to the user
        println!("\nCSV Output:\n");

        // Read and display a preview of the temporary file
        let (content, line_count) = writer.preview_file(&temp_path)?;
        println!("{}...\n[{} rows total]", content, line_count);

        // Clean up the temporary file
        std::fs::remove_file(temp_path)?;

        // Show the user how to save to a file in the future
        println!("\nTo save this output to a file, run the command with the -o flag:");
        println!("moxfield-to-csv \"{}\" -o deck.csv", deck_url);
    }

    // Return success
    Ok(())
}
