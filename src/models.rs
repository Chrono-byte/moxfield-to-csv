// Import Serde's Deserialize trait for JSON parsing
use serde::Deserialize;

/// Represents a Magic: The Gathering card in the Moxfield API response
///
/// This structure contains the basic card information needed for our purposes
#[derive(Debug, Deserialize)]
pub struct Card {
    /// The name of the card
    pub name: String,
}

/// Represents a card entry in a deck, including both the card and its quantity
#[derive(Debug, Deserialize)]
pub struct CardEntry {
    /// The card object containing the card's details
    pub card: Card,
    /// The number of copies of this card in the deck
    pub quantity: i32,
}

/// The main response structure from the Moxfield API
///
/// Contains separate mappings for commanders and main deck cards
#[derive(Debug, Deserialize)]
pub struct MoxfieldResponse {
    /// The name of the deck
    pub name: String,
    /// Map of commander entries, keyed by their IDs in Moxfield's system
    pub commanders: std::collections::HashMap<String, CardEntry>,
    /// Map of main deck entries, keyed by their IDs in Moxfield's system
    pub mainboard: std::collections::HashMap<String, CardEntry>,
}

/// A simplified representation of a deck after processing
///
/// This structure contains just the essential information needed for CSV export
#[derive(Debug)]
pub struct ProcessedDeck {
    /// The name of the deck
    pub name: String,
    /// Vector of (card_name, quantity) pairs for the main deck
    pub main_deck: Vec<(String, i32)>,
    /// Vector of commander card names
    pub commanders: Vec<String>,
}

impl ProcessedDeck {
    /// Creates a new, empty ProcessedDeck
    ///
    /// Initializes with empty vectors for both main deck and commanders
    pub fn new() -> Self {
        Self {
            name: String::new(),
            main_deck: Vec::new(),
            commanders: Vec::new(),
        }
    }
}

/// Error handling module for the application
/// Error handling module for the application
pub mod error {
    use std::fmt;

    /// Represents all possible errors that can occur in the application
    #[derive(Debug)]
    pub enum Error {
        /// Errors that occur during network operations
        Network(reqwest::Error),
        /// Errors that occur while parsing data (URLs, JSON, etc.)
        Parse(String),
        /// Errors that occur during file I/O operations
        Io(std::io::Error),
        /// Any other errors that don't fit into the above categories
        Other(String),
    }

    // Implement the standard error trait for our Error enum
    impl std::error::Error for Error {}

    /// Implements string formatting for our Error type
    impl fmt::Display for Error {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Error::Network(e) => write!(f, "Network error: {}", e),
                Error::Parse(e) => write!(f, "Parse error: {}", e),
                Error::Io(e) => write!(f, "IO error: {}", e),
                Error::Other(e) => write!(f, "{}", e),
            }
        }
    }

    /// Automatically convert reqwest errors into our Error type
    impl From<reqwest::Error> for Error {
        fn from(err: reqwest::Error) -> Self {
            Error::Network(err)
        }
    }

    /// Automatically convert IO errors into our Error type
    impl From<std::io::Error> for Error {
        fn from(err: std::io::Error) -> Self {
            Error::Io(err)
        }
    }
}
