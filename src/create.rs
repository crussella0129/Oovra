//! Create: turn plain Markdown files into Oovra atoms.
//!
//! Two file-based modes (the CLI picks exactly one):
//!
//! - **label** — prepend a header to a file *in place*. The file itself
//!   becomes the element; no plain copy is left behind.
//! - **olib** — write a headered *copy* into an `olib/` library directory,
//!   leaving the original untouched. An input that is already an Oovra
//!   file is copied verbatim (no second header), which makes olib-to-olib
//!   transfer a side effect of the same mode.
//!
//! Both modes verify their work by re-parsing the output through the
//! Stage 1 parser; a broken file is reported, never left on disk.

use std::fs;
use std::path::{Path, PathBuf};

use crate::element::{looks_like_oovra_file, parse, parse_file, write, PromptElement};
use crate::error::{OovraError, Result};
use crate::header::{PromptElementHeader, PromptElementKind};

/// Build the header for a freshly labeled atom. The element name defaults
/// to the id — file-based authoring doesn't carry a separate display name.
fn atom_header(id: &str, version: &str, meta: &str) -> PromptElementHeader {
    PromptElementHeader {
        name: id.to_string(),
        kind: PromptElementKind::Atom,
        id: id.to_string(),
        version: version.to_string(),
        meta: meta.to_string(),
        generated_at: None,
        render_mode: None,
        body_level: None,
        depth: None,
        composed_of: None,
    }
}

/// Choose the body for a labeled element: the file's content verbatim, or
/// a TODO placeholder when the source was empty.
fn body_or_placeholder(content: &str, id: &str) -> String {
    if content.trim().is_empty() {
        format!("<!-- TODO: body for `{id}` was empty when labeled -->")
    } else {
        content.to_string()
    }
}

/// Prepend a header to `path` **in place** — the file becomes the element.
///
/// `content` is the file's current text (already read by the caller). If
/// the file is already an Oovra file this refuses unless `force`, in which
/// case the existing header is peeled off and replaced.
pub fn label_in_place(
    path: &Path,
    content: &str,
    id: &str,
    version: &str,
    meta: &str,
    force: bool,
) -> Result<PathBuf> {
    let already = looks_like_oovra_file(content);
    if already && !force {
        return Err(OovraError::AlreadyLabeled(path.to_path_buf()));
    }

    let body = if already {
        // Force-relabel: peel the old frontmatter, or keep the whole file
        // verbatim if the format is too non-standard to split cleanly.
        peel_existing_frontmatter(content).unwrap_or_else(|| content.to_string())
    } else {
        body_or_placeholder(content, id)
    };

    let element = PromptElement::new(atom_header(id, version, meta), body);
    write(&element, path)?;
    Ok(path.to_path_buf())
}

/// Write a headered *copy* of a plain Markdown file into `olib_dir/<id>.md`.
/// The original file is never touched.
pub fn label_into_olib(
    olib_dir: &Path,
    content: &str,
    id: &str,
    version: &str,
    meta: &str,
) -> Result<PathBuf> {
    let element = PromptElement::new(
        atom_header(id, version, meta),
        body_or_placeholder(content, id),
    );
    let dest = olib_dir.join(format!("{id}.md"));
    write(&element, &dest)?;
    Ok(dest)
}

/// Copy an input that is *already* an Oovra file into `olib_dir`, verbatim.
///
/// The content is validated by a full parse first (so garbage cannot enter
/// the library), then written byte-for-byte to `olib_dir/<id>.md` named
/// after its own header id. This is what makes `--olib` double as an
/// olib-to-olib transfer: an already-headered input keeps its single
/// header instead of gaining a second one.
pub fn copy_oovra_into_olib(olib_dir: &Path, source: &Path, content: &str) -> Result<PathBuf> {
    let element = parse(content, source)?;
    ensure_dir(olib_dir)?;
    let dest = olib_dir.join(format!("{}.md", element.header.id));
    fs::write(&dest, content).map_err(|source| OovraError::WriteIo {
        path: dest.clone(),
        source,
    })?;
    // Paranoia: re-parse what landed on disk.
    parse_file(&dest)?;
    Ok(dest)
}

/// Best-effort: strip the first `+++ ... +++` frontmatter block. Returns
/// None if the format is non-standard.
fn peel_existing_frontmatter(content: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()?.trim_end() != "+++" {
        return None;
    }
    let mut body_lines: Vec<&str> = Vec::new();
    let mut found_close = false;
    for line in lines {
        if !found_close {
            if line.trim_end() == "+++" {
                found_close = true;
            }
        } else {
            body_lines.push(line);
        }
    }
    if !found_close {
        return None;
    }
    // Skip exactly one blank line after the closing delimiter.
    let start = if body_lines
        .first()
        .map(|l| l.trim().is_empty())
        .unwrap_or(false)
    {
        1
    } else {
        0
    };
    Some(body_lines[start..].join("\n"))
}

/// Convenience: ensure the directory exists.
pub fn ensure_dir(p: &Path) -> Result<()> {
    fs::create_dir_all(p).map_err(|source| OovraError::WriteIo {
        path: p.to_path_buf(),
        source,
    })
}
