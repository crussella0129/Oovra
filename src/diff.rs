//! Compare: the FORWARD-DIFF operator with order-aware dispatch.
//!
//! - **Same order, both 0** → content diff (body unified diff + metadata
//!   changes).
//! - **Same order, both >= 1** → structural diff over `composed_of`. Reports
//!   inputs added, removed, and version-changed.
//! - **Different orders** → refused with a clear error suggesting the user
//!   probably meant to compare same-order elements.

use std::collections::HashMap;

use serde::Serialize;
use similar::TextDiff;

use crate::element::PromptElement;
use crate::error::{OovraError, Result};
use crate::header::InputRef;

/// Top-level result of [`compare`].
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DiffReport {
    Content(ContentDiff),
    Structural(StructuralDiff),
}

/// Diff of two atomic (order-0) elements.
#[derive(Debug, Serialize)]
pub struct ContentDiff {
    pub a_id: String,
    pub b_id: String,
    pub field_changes: Vec<FieldChange>,
    pub body_unified_diff: String,
    pub bodies_equal: bool,
}

#[derive(Debug, Serialize)]
pub struct FieldChange {
    pub field: String,
    pub before: String,
    pub after: String,
}

/// Diff of two composed (order >= 1) elements.
#[derive(Debug, Serialize)]
pub struct StructuralDiff {
    pub a_id: String,
    pub b_id: String,
    pub order: u32,
    pub added: Vec<InputRef>,
    pub removed: Vec<InputRef>,
    pub version_changed: Vec<VersionChange>,
    pub recipes_equal: bool,
}

#[derive(Debug, Serialize)]
pub struct VersionChange {
    pub id: String,
    pub before_version: String,
    pub after_version: String,
}

/// Dispatch a comparison between two prompt elements based on their orders
/// and whether each has a recipe.
pub fn compare(a: &PromptElement, b: &PromptElement) -> Result<DiffReport> {
    if a.header.order != b.header.order {
        return Err(OovraError::OrderMismatch {
            a_id: a.header.id.clone(),
            a_order: a.header.order,
            b_id: b.header.id.clone(),
            b_order: b.header.order,
        });
    }

    // Both must be either atomic or both composed; mixing is ambiguous.
    match (a.header.is_atomic(), b.header.is_atomic()) {
        (true, true) => Ok(DiffReport::Content(content_diff(a, b))),
        (false, false) => Ok(DiffReport::Structural(structural_diff(a, b)?)),
        (a_atomic, _) => Err(OovraError::AtomicityMismatch {
            a_id: a.header.id.clone(),
            a_kind: if a_atomic { "atomic" } else { "composed" },
            b_id: b.header.id.clone(),
            b_kind: if a_atomic { "composed" } else { "atomic" },
        }),
    }
}

fn content_diff(a: &PromptElement, b: &PromptElement) -> ContentDiff {
    let mut field_changes = Vec::new();

    track_field(&mut field_changes, "name", &a.header.name, &b.header.name);
    track_field(
        &mut field_changes,
        "version",
        &a.header.version,
        &b.header.version,
    );
    track_field(&mut field_changes, "meta", &a.header.meta, &b.header.meta);

    let diff = TextDiff::from_lines(&a.body, &b.body);
    let body_unified_diff = diff
        .unified_diff()
        .header(&a.header.id, &b.header.id)
        .to_string();
    let bodies_equal = a.body == b.body;

    ContentDiff {
        a_id: a.header.id.clone(),
        b_id: b.header.id.clone(),
        field_changes,
        body_unified_diff,
        bodies_equal,
    }
}

fn track_field(out: &mut Vec<FieldChange>, name: &str, before: &str, after: &str) {
    if before != after {
        out.push(FieldChange {
            field: name.to_string(),
            before: before.to_string(),
            after: after.to_string(),
        });
    }
}

fn structural_diff(a: &PromptElement, b: &PromptElement) -> Result<StructuralDiff> {
    let a_inputs = a.header.composed_of.as_ref().ok_or_else(|| {
        OovraError::OrderRequiresField {
            id: a.header.id.clone(),
            order: a.header.order,
            field: "composed_of",
        }
    })?;
    let b_inputs = b.header.composed_of.as_ref().ok_or_else(|| {
        OovraError::OrderRequiresField {
            id: b.header.id.clone(),
            order: b.header.order,
            field: "composed_of",
        }
    })?;

    let a_by_id: HashMap<&str, &str> = a_inputs
        .iter()
        .map(|i| (i.id.as_str(), i.version.as_str()))
        .collect();
    let b_by_id: HashMap<&str, &str> = b_inputs
        .iter()
        .map(|i| (i.id.as_str(), i.version.as_str()))
        .collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut version_changed = Vec::new();

    for input in b_inputs {
        match a_by_id.get(input.id.as_str()) {
            Some(a_ver) if *a_ver != input.version.as_str() => {
                version_changed.push(VersionChange {
                    id: input.id.clone(),
                    before_version: a_ver.to_string(),
                    after_version: input.version.clone(),
                });
            }
            None => added.push(input.clone()),
            _ => {}
        }
    }

    for input in a_inputs {
        if !b_by_id.contains_key(input.id.as_str()) {
            removed.push(input.clone());
        }
    }

    let recipes_equal =
        added.is_empty() && removed.is_empty() && version_changed.is_empty();

    Ok(StructuralDiff {
        a_id: a.header.id.clone(),
        b_id: b.header.id.clone(),
        order: a.header.order,
        added,
        removed,
        version_changed,
        recipes_equal,
    })
}
