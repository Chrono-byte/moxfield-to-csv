// Import error handling and data structures from our models module
use crate::models::{MoxfieldResponse, ProcessedDeck, error::Error};
// Import HTTP client from reqwest crate
use reqwest::Client;
// Import URL parsing functionality
use url::Url;

/// The base URL for the Moxfield API's deck endpoint
const API_BASE: &str = "https://api.moxfield.com/v2/decks/all";

/// A client for interacting with the Moxfield API
///
/// This struct wraps an HTTP client and provides methods for
/// fetching deck information from Moxfield
pub struct MoxfieldClient {
    /// The underlying HTTP client used for making requests
    client: Client,
}

impl MoxfieldClient {
    /// Creates a new instance of MoxfieldClient with default configuration
    ///
    /// The client is configured without gzip compression to simplify response handling
    pub fn new() -> Result<Self, Error> {
        // Create a new HTTP client with default configuration. Return an
        // application Error if building the client fails so callers can
        // handle it instead of silently falling back.
        let client = Client::builder()
            // Keep default compression handling; disabling gzip is usually
            // unnecessary and may reduce compatibility.
            .build()
            .map_err(Error::Network)?;

        Ok(Self { client })
    }

    /// Fetches a deck from Moxfield by its ID
    ///
    /// # Arguments
    /// * `deck_id` - The unique identifier of the deck on Moxfield
    ///
    /// # Returns
    /// A Result containing either the processed deck data or an error
    pub async fn fetch_deck(&self, deck_id: &str) -> Result<ProcessedDeck, Error> {
        // Construct the full API URL for the deck
        let url = format!("{}/{}", API_BASE, deck_id);

        // Make the HTTP GET request to the Moxfield API
        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .header("User-Agent", "Moxfield-CSV-Exporter/Rust") // Identify our application
            .send()
            .await?;

        // Check if the request was successful
        if !response.status().is_success() {
            return Err(Error::Other(format!(
                "Failed to fetch deck: {}",
                response.status()
            )));
        }

        // Parse the JSON response into our data structure
        let moxfield_data: MoxfieldResponse = response.json().await?;
        // Create a new empty processed deck to store our results
        let mut processed_deck = ProcessedDeck::new();
        
        // Set the deck name
        processed_deck.name = moxfield_data.name;

        // Extract commander information
        // Map each commander entry to just its name and collect into a Vec
        processed_deck.commanders = moxfield_data
            .commanders
            .values()
            .map(|entry| entry.card.name.clone())
            .collect();

        // throw if there are more than two commanders
        if processed_deck.commanders.len() > 2 {
            return Err(Error::Parse("Deck has more than two commanders".to_string()));
        }

        // Process the main deck cards
        // Use a HashMap for combining duplicate cards' quantities
        let mut card_counts = std::collections::HashMap::new();
        for entry in moxfield_data.mainboard.values() {
            // Add the quantity to existing count or insert new count
            *card_counts.entry(entry.card.name.clone()).or_insert(0) += entry.quantity;
        }

        // Convert the HashMap to a sorted Vec for consistent output
        let mut main_deck: Vec<_> = card_counts.into_iter().collect();
        main_deck.sort_by(|a, b| a.0.cmp(&b.0)); // Sort by card name
        processed_deck.main_deck = main_deck;

        Ok(processed_deck)
    }

    /// Extracts a deck ID from a Moxfield URL or validates a raw deck ID
    ///
    /// # Arguments
    /// * `url` - A string that could be either a full Moxfield URL or just a deck ID
    ///
    /// # Returns
    /// * `Some(String)` - The extracted deck ID if valid
    /// * `None` - If the input is neither a valid URL nor a valid deck ID
    pub fn extract_deck_id(url: &str) -> Option<String> {
        // Fast path - Check if the input is already just a deck ID
        // Valid IDs contain only alphanumeric characters, hyphens, and underscores
        if url
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Some(url.to_string());
        }

        // If not a raw ID, attempt to parse as a URL
        let url = Url::parse(url).ok()?;
        // Verify this is a Moxfield URL
        if !url.host_str()?.contains("moxfield.com") {
            return None;
        }

        // Split the URL path into segments and collect them
        let path_segments: Vec<_> = url.path_segments()?.collect();
        // Check if this is a deck URL (should start with "decks")
        if path_segments.get(0) == Some(&"decks") {
            // If so, return the second segment (the deck ID)
            path_segments.get(1).map(|&id| id.to_string())
        } else {
            None
        }
    }
}
