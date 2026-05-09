//! Oovra: a tool for the composition and comparison of agentic system
//! prompts from Markdown+TOML "prompt elements".
//!
//! The library exposes the parser, library loader, and the four operators —
//! Create, Compose, Decompose, Compare — for use by the `oovra` binary or
//! any other Rust consumer.

pub mod create;
pub mod decompose;
pub mod diff;
pub mod element;
pub mod error;
pub mod header;
pub mod library;
pub mod render;

pub use element::{parse, parse_file, serialize, write, PromptElement};
pub use error::{OovraError, Result};
pub use header::{InputRef, PromptElementHeader};
pub use library::Library;
