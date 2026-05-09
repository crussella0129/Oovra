//! Parsing, validation, and serialization of complete Oovra files.
//!
//! An Oovra file is exactly:
//!
//! ```text
//! +++
//! <TOML frontmatter>
//! +++
//!
//! <Markdown body>
//! ```
//!
//! For order-0 elements the body is freeform Markdown. For order-N (N>=1)
//! elements the body is a sequence of K complete sub-element files,
//! delimited by chiral, order-aware tilde lines:
//!
//! - **Open** for level N: a line containing exactly `~~...~~>>` with `(N+1)`
//!   tilde characters.
//! - **Close** for level N: a line containing exactly `~~...~~<<` with `(N+1)`
//!   tilde characters.
//!
//! The escalation rule: a level-N delimiter has strictly more tildes than any
//! level less than N, so an outer parser scanning for level-N delimiters
//! ignores inner level-(N-k) delimiters by tilde count. The `>>` / `<<`
//! suffix gives chirality so opens and closes can never be confused.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{OovraError, Result};
use crate::header::{
    is_kebab_case, is_valid_rfc3339, is_valid_semver, PromptElementHeader,
};

/// In-memory representation of one Oovra file.
#[derive(Debug, Clone)]
pub struct PromptElement {
    pub header: PromptElementHeader,
    pub body: String,
    pub source_path: Option<PathBuf>,
}

impl PromptElement {
    pub fn new(header: PromptElementHeader, body: String) -> Self {
        Self {
            header,
            body,
            source_path: None,
        }
    }
}

/// Open delimiter for level-`body_level` body splitting. Uses
/// `(body_level + 1)` tildes plus the chiral `>>` suffix.
pub fn body_delimiter_open(body_level: u32) -> String {
    let tildes = "~".repeat((body_level + 1) as usize);
    format!("{tildes}>>")
}

/// Close delimiter for level-`body_level` body splitting. Symmetric to
/// [`body_delimiter_open`] but with the chiral `<<` suffix.
pub fn body_delimiter_close(body_level: u32) -> String {
    let tildes = "~".repeat((body_level + 1) as usize);
    format!("{tildes}<<")
}

/// Split a file's content into (frontmatter_string, body_string).
///
/// Rules:
/// - The first line must be exactly `+++` (trailing whitespace is tolerated
///   but the line must contain only the delimiter).
/// - The next line containing exactly `+++` (and no other characters) closes
///   the frontmatter.
/// - Exactly one blank line after the closing `+++` is consumed; the rest of
///   the file is the body.
pub fn split_frontmatter(content: &str, path: &Path) -> Result<(String, String)> {
    let mut lines = content.lines();

    let first = lines.next();
    match first {
        Some(line) if line.trim_end() == "+++" => {}
        Some(line) => {
            return Err(OovraError::MissingOpenDelimiter {
                path: path.to_path_buf(),
                actual: line.to_string(),
            });
        }
        None => return Err(OovraError::EmptyFile(path.to_path_buf())),
    }

    let mut fm_lines: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();
    let mut closed = false;

    for line in lines {
        if !closed {
            if line.trim_end() == "+++" {
                closed = true;
            } else {
                fm_lines.push(line);
            }
        } else {
            body_lines.push(line);
        }
    }

    if !closed {
        return Err(OovraError::MissingCloseDelimiter(path.to_path_buf()));
    }

    // Consume exactly one blank line after the closing delimiter, if present.
    let body_start = if body_lines.first().map(|l| l.trim().is_empty()).unwrap_or(false) {
        1
    } else {
        0
    };

    let body = body_lines[body_start..].join("\n");
    let frontmatter = fm_lines.join("\n");

    Ok((frontmatter, body))
}

/// Parse a complete Oovra file from a string. Validates the header semantics.
pub fn parse(content: &str, path: &Path) -> Result<PromptElement> {
    let (fm_str, body) = split_frontmatter(content, path)?;

    let header: PromptElementHeader =
        toml::from_str(&fm_str).map_err(|source| OovraError::InvalidToml {
            path: path.to_path_buf(),
            source,
        })?;

    validate_header(&header, &body, path)?;

    Ok(PromptElement {
        header,
        body,
        source_path: Some(path.to_path_buf()),
    })
}

/// Read and parse a file from disk.
pub fn parse_file(path: &Path) -> Result<PromptElement> {
    if !path.exists() {
        return Err(OovraError::FileNotFound(path.to_path_buf()));
    }
    let content = fs::read_to_string(path).map_err(|source| OovraError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    parse(&content, path)
}

fn validate_header(header: &PromptElementHeader, body: &str, path: &Path) -> Result<()> {
    if !is_kebab_case(&header.id) {
        return Err(OovraError::InvalidField {
            path: path.to_path_buf(),
            field: "id",
            value: header.id.clone(),
            reason: "must be kebab-case (lowercase letters, digits, hyphens; no leading/trailing/double hyphens)".to_string(),
        });
    }

    if !is_valid_semver(&header.version) {
        return Err(OovraError::InvalidField {
            path: path.to_path_buf(),
            field: "version",
            value: header.version.clone(),
            reason: "must be valid semver (e.g. \"1.0.0\")".to_string(),
        });
    }

    if header.name.trim().is_empty() {
        return Err(OovraError::InvalidField {
            path: path.to_path_buf(),
            field: "name",
            value: header.name.clone(),
            reason: "must be non-empty".to_string(),
        });
    }

    if body.trim().is_empty() {
        return Err(OovraError::EmptyBody(path.to_path_buf()));
    }

    if header.is_composed() {
        // composed_of is Some — require the companion fields jointly.
        let generated_at = header.generated_at.as_deref().ok_or_else(|| {
            OovraError::OrderRequiresField {
                id: header.id.clone(),
                order: header.order,
                field: "generated_at",
            }
        })?;
        if !is_valid_rfc3339(generated_at) {
            return Err(OovraError::InvalidField {
                path: path.to_path_buf(),
                field: "generated_at",
                value: generated_at.to_string(),
                reason: "must be RFC 3339 (e.g. \"2026-05-09T14:23:15Z\")".to_string(),
            });
        }

        if header.render_mode.is_none() {
            return Err(OovraError::OrderRequiresField {
                id: header.id.clone(),
                order: header.order,
                field: "render_mode",
            });
        }

        if header.body_level.is_none() {
            return Err(OovraError::OrderRequiresField {
                id: header.id.clone(),
                order: header.order,
                field: "body_level",
            });
        }

        let composed_of = header.composed_of.as_ref().expect("is_composed checked");

        if composed_of.is_empty() {
            return Err(OovraError::InvalidField {
                path: path.to_path_buf(),
                field: "composed_of",
                value: "[]".to_string(),
                reason: "composed elements must have at least one input".to_string(),
            });
        }

        for input in composed_of {
            if !is_kebab_case(&input.id) {
                return Err(OovraError::InvalidField {
                    path: path.to_path_buf(),
                    field: "composed_of[].id",
                    value: input.id.clone(),
                    reason: "must be kebab-case".to_string(),
                });
            }
            if !is_valid_semver(&input.version) {
                return Err(OovraError::InvalidField {
                    path: path.to_path_buf(),
                    field: "composed_of[].version",
                    value: input.version.clone(),
                    reason: "must be valid semver".to_string(),
                });
            }
        }
    } else {
        // composed_of is None: this must be an order-0 hand-authored
        // element, and all companion fields must also be absent.
        if header.order != 0 {
            return Err(OovraError::HandAuthoredHigherOrder {
                path: path.to_path_buf(),
                order: header.order,
            });
        }
        if header.generated_at.is_some()
            || header.render_mode.is_some()
            || header.body_level.is_some()
        {
            return Err(OovraError::InvalidField {
                path: path.to_path_buf(),
                field: "generated_at|render_mode|body_level",
                value: "<set>".to_string(),
                reason: "these fields are only valid when composed_of is also present"
                    .to_string(),
            });
        }
    }

    Ok(())
}

/// Serialize a [`PromptElement`] to its on-disk string form.
///
/// Output: `+++\n<toml>+++\n\n<body>\n` (trailing newline always present).
pub fn serialize(element: &PromptElement) -> Result<String> {
    let toml_string =
        toml::to_string_pretty(&element.header).map_err(|source| OovraError::TomlSerialize {
            id: element.header.id.clone(),
            source,
        })?;

    let body_trimmed = element.body.trim_end_matches('\n');
    Ok(format!("+++\n{toml_string}+++\n\n{body_trimmed}\n"))
}

/// Write a [`PromptElement`] to disk.
///
/// Validates the serialized form *in memory* before any disk write happens —
/// so an invalid element (bad id, missing required field, etc.) never
/// produces a partial file on disk. After the write, parses the file back
/// as a paranoia check against filesystem-layer corruption (encoding,
/// line-endings).
pub fn write(element: &PromptElement, path: &Path) -> Result<()> {
    let content = serialize(element)?;
    // Validate first by parsing the in-memory string. If this fails,
    // nothing is touched on disk.
    let _ = parse(&content, path)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|source| OovraError::WriteIo {
                path: parent.to_path_buf(),
                source,
            })?;
        }
    }
    fs::write(path, &content).map_err(|source| OovraError::WriteIo {
        path: path.to_path_buf(),
        source,
    })?;
    parse_file(path)?;
    Ok(())
}

/// Detect whether a string starts with the Oovra `+++` opening delimiter.
pub fn looks_like_oovra_file(content: &str) -> bool {
    content.starts_with("+++\n") || content.starts_with("+++\r\n") || content.trim_start() == "+++"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::InputRef;

    #[test]
    fn delimiter_scales_with_order() {
        assert_eq!(body_delimiter_open(1), "~~>>");
        assert_eq!(body_delimiter_close(1), "~~<<");
        assert_eq!(body_delimiter_open(2), "~~~>>");
        assert_eq!(body_delimiter_close(2), "~~~<<");
        assert_eq!(body_delimiter_open(5), "~~~~~~>>");
        assert_eq!(body_delimiter_close(5), "~~~~~~<<");
    }

    #[test]
    fn split_frontmatter_minimal_node() {
        let content = "+++\nname = \"Test\"\norder = 0\nid = \"test\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nThe body.\n";
        let (fm, body) = split_frontmatter(content, Path::new("test.md")).unwrap();
        assert!(fm.contains("name = \"Test\""));
        assert_eq!(body, "The body.");
    }

    #[test]
    fn split_frontmatter_rejects_missing_open() {
        let content = "name = \"Test\"\n+++\n\nbody";
        let err = split_frontmatter(content, Path::new("test.md")).unwrap_err();
        assert!(matches!(err, OovraError::MissingOpenDelimiter { .. }));
    }

    #[test]
    fn split_frontmatter_rejects_missing_close() {
        let content = "+++\nname = \"Test\"\nno close here\n";
        let err = split_frontmatter(content, Path::new("test.md")).unwrap_err();
        assert!(matches!(err, OovraError::MissingCloseDelimiter(_)));
    }

    #[test]
    fn parse_round_trips_minimal_node() {
        let content = "+++\nname = \"Refusal Policy\"\norder = 0\nid = \"refusal-policy\"\nversion = \"1.0.0\"\nmeta = \"Be brief.\"\n+++\n\nDecline harmful requests briefly.\n";
        let element = parse(content, Path::new("refusal.md")).unwrap();
        assert_eq!(element.header.id, "refusal-policy");
        assert_eq!(element.header.order, 0);
        assert_eq!(element.body, "Decline harmful requests briefly.");

        let serialized = serialize(&element).unwrap();
        let parsed_again = parse(&serialized, Path::new("refusal.md")).unwrap();
        assert_eq!(parsed_again.header.id, element.header.id);
        assert_eq!(parsed_again.body, element.body);
    }

    #[test]
    fn parse_rejects_non_kebab_id() {
        let content = "+++\nname = \"X\"\norder = 0\nid = \"NotKebab\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nbody\n";
        let err = parse(content, Path::new("x.md")).unwrap_err();
        assert!(matches!(err, OovraError::InvalidField { field: "id", .. }));
    }

    #[test]
    fn parse_rejects_non_semver_version() {
        let content = "+++\nname = \"X\"\norder = 0\nid = \"x\"\nversion = \"v1.0\"\nmeta = \"\"\n+++\n\nbody\n";
        let err = parse(content, Path::new("x.md")).unwrap_err();
        assert!(matches!(err, OovraError::InvalidField { field: "version", .. }));
    }

    #[test]
    fn parse_rejects_higher_order_without_composed_of() {
        // A hand-authored claim of `order = 1` without a recipe is rejected.
        let content = "+++\nname = \"X\"\norder = 1\nid = \"x\"\nversion = \"1.0.0\"\nmeta = \"\"\n+++\n\nbody\n";
        let err = parse(content, Path::new("x.md")).unwrap_err();
        assert!(matches!(err, OovraError::HandAuthoredHigherOrder { .. }));
    }

    #[test]
    fn parse_accepts_valid_composed_element() {
        let content = "+++\nname = \"Composed\"\norder = 1\nid = \"composed\"\nversion = \"1.0.0\"\nmeta = \"\"\ngenerated_at = \"2026-05-09T14:23:15Z\"\nrender_mode = \"markdown-h2\"\nbody_level = 1\ncomposed_of = [{id = \"a\", version = \"1.0.0\"}, {id = \"b\", version = \"1.0.0\"}]\n+++\n\nbody\n";
        let element = parse(content, Path::new("composed.md")).unwrap();
        assert_eq!(element.header.order, 1);
        assert_eq!(element.header.body_level, Some(1));
        let composed_of = element.header.composed_of.as_ref().unwrap();
        assert_eq!(composed_of.len(), 2);
        assert_eq!(composed_of[0], InputRef::new("a", "1.0.0"));
    }

    #[test]
    fn parse_rejects_atomic_with_companion_fields() {
        // Order-0 file with composed-only fields set should be rejected.
        let content = "+++\nname = \"X\"\norder = 0\nid = \"x\"\nversion = \"1.0.0\"\nmeta = \"\"\nbody_level = 1\n+++\n\nbody\n";
        let err = parse(content, Path::new("x.md")).unwrap_err();
        assert!(matches!(err, OovraError::InvalidField { .. }));
    }

    #[test]
    fn parse_rejects_composed_without_body_level() {
        let content = "+++\nname = \"X\"\norder = 1\nid = \"x\"\nversion = \"1.0.0\"\nmeta = \"\"\ngenerated_at = \"2026-05-09T14:23:15Z\"\nrender_mode = \"markdown-h2\"\ncomposed_of = [{id = \"a\", version = \"1.0.0\"}]\n+++\n\nbody\n";
        let err = parse(content, Path::new("x.md")).unwrap_err();
        assert!(matches!(err, OovraError::OrderRequiresField { field: "body_level", .. }));
    }
}
