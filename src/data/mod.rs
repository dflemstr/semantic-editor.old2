//! Definitions of data types that can be edited.
//!
//! Here's where you would add support for new file types and formats.
pub mod markdown;

/// Some data that can be edited.
#[derive(Debug, Semantic)]
#[semantic(role = "root")]
pub enum Data {
    /// The `Markdown` variant.
    Markdown(markdown::Markdown),
}
