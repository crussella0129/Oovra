//! Error type for Oovra operations.
//!
//! Error messages are part of the agent-facing API: every variant attaches the
//! file path and explains specifically what went wrong. An LLM agent reading
//! "Field 'version' missing in nodes/refusal.md" can act; "TOML parse failed"
//! cannot.

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OovraError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to read {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to write {path}: {source}")]
    WriteIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Empty file: {0}")]
    EmptyFile(PathBuf),

    #[error("Missing opening '+++' delimiter on line 1 of {path}. Expected '+++', got '{actual}'.")]
    MissingOpenDelimiter { path: PathBuf, actual: String },

    #[error("Missing closing '+++' delimiter in {0}. Frontmatter must be terminated by '+++' on its own line.")]
    MissingCloseDelimiter(PathBuf),

    #[error("Invalid TOML in frontmatter of {path}: {source}")]
    InvalidToml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to serialize TOML for element '{id}': {source}")]
    TomlSerialize {
        id: String,
        #[source]
        source: toml::ser::Error,
    },

    #[error("Missing required field '{field}' in {path}")]
    MissingField { path: PathBuf, field: &'static str },

    #[error("Field '{field}' in {path} has invalid value '{value}': {reason}")]
    InvalidField {
        path: PathBuf,
        field: &'static str,
        value: String,
        reason: String,
    },

    #[error("Empty body in {0}. The body must be non-empty after stripping whitespace.")]
    EmptyBody(PathBuf),

    #[error("Order-{order} element '{id}' is missing the '{field}' field, which is required for order >= 1")]
    OrderRequiresField {
        id: String,
        order: u32,
        field: &'static str,
    },

    #[error("Hand-authored elements must be order 0; found order {order} in {path}. Use 'oovra compose' to produce higher-order elements.")]
    HandAuthoredHigherOrder { path: PathBuf, order: u32 },

    #[error("Duplicate ID '{id}' in library: '{first}' and '{second}'")]
    DuplicateId {
        id: String,
        first: PathBuf,
        second: PathBuf,
    },

    #[error("Element '{id}' not found in library")]
    ElementNotFound { id: String },

    #[error("Version mismatch for '{id}': pin '{pin}' does not match library version '{actual}'")]
    VersionMismatch {
        id: String,
        pin: String,
        actual: String,
    },

    #[error("Cannot compare elements of different orders: '{a_id}' is order {a_order}, '{b_id}' is order {b_order}. Compare requires same-order inputs.")]
    OrderMismatch {
        a_id: String,
        a_order: u32,
        b_id: String,
        b_order: u32,
    },

    #[error("Cannot compare an atomic element with a composed element: '{a_id}' is {a_kind}, '{b_id}' is {b_kind}. Compare requires both inputs to be the same kind (both atomic or both composed).")]
    AtomicityMismatch {
        a_id: String,
        a_kind: &'static str,
        b_id: String,
        b_kind: &'static str,
    },

    #[error("Compose requires at least one input")]
    EmptyCompose,

    #[error("File {0} already has an Oovra header. Use --force to overwrite.")]
    AlreadyLabeled(PathBuf),

    #[error("Cannot decompose atomic element '{id}'. Atomic elements have no recipe (no `composed_of` field). Only Compose-produced elements can be decomposed.")]
    CannotDecomposeAtomic { id: String },

    #[error("Body of order-{order} element '{id}' could not be split into the expected sub-element chunks: {reason}")]
    BodyParse {
        id: String,
        order: u32,
        reason: String,
    },

    #[error("'{0}' is not a directory")]
    NotADirectory(PathBuf),
}

pub type Result<T> = std::result::Result<T, OovraError>;
