//! Compose: the JOIN operator.
//!
//! Compose takes an ordered list of input prompt elements and produces a
//! single composed prompt element of higher (or equal) order.
//!
//! The output's order is computed by [`compute_order`]: if at least two
//! inputs share the maximum input order, the output is one order higher;
//! otherwise the output keeps the maximum order. This encodes "compositional
//! depth" — you only climb when you genuinely peer-compose.
//!
//! The output's body is the concatenation of each input's *full file content*
//! (frontmatter + body), wrapped in chiral order-aware delimiters. This makes
//! every composed file lossless: `decompose --full` recovers every element
//! at every level by recursively splitting bodies.

use chrono::Utc;

use crate::element::{
    body_delimiter_close, body_delimiter_open, serialize, PromptElement,
};
use crate::error::{OovraError, Result};
use crate::header::{InputRef, PromptElementHeader, PromptElementKind};
use crate::library::Library;

/// Inputs to a Compose operation.
pub struct ComposeRequest<'a> {
    pub library: &'a Library,
    /// Each entry: (id, optional version pin). When the pin is `Some`, the
    /// library's version of the element must match exactly.
    pub inputs: Vec<(String, Option<String>)>,
    pub output_id: String,
    pub output_name: String,
    pub output_version: String,
    pub output_meta: String,
}

/// Compute the **logical order** of a Compose output from the orders of its
/// inputs.
///
/// Rule (from the v0.1 spec):
///
/// ```text
/// let H = max(input.order for input in inputs)
/// let C = count of inputs with order == H
/// output_order = if C > 1 { H + 1 } else { H }
/// ```
///
/// This means: composing N order-0 elements (with N >= 2) yields order 1.
/// Composing two order-1 elements yields order 2. Composing one order-1 and
/// many order-0 elements stays at order 1 — the order-1 has no peer at its
/// level, so no climb happens.
///
/// Note: this is the *logical* depth. The on-disk delimiter level uses
/// [`compute_body_level`] instead, which is always strictly greater than
/// any input's body delimiter level. The two values coincide for the
/// homogeneous case but diverge when `count_at_highest == 1`.
pub fn compute_order(orders: &[u32]) -> u32 {
    if orders.is_empty() {
        return 0;
    }
    let highest = orders.iter().copied().max().unwrap_or(0);
    let count_at_highest = orders.iter().filter(|&&o| o == highest).count();
    if count_at_highest > 1 {
        highest + 1
    } else {
        highest
    }
}

/// Compute the **physical body delimiter level** for a Compose output.
///
/// Always `max(input.order) + 1`, regardless of whether the logical order
/// climbs. This is what makes the body parser unambiguous: the outer
/// delimiter has strictly more tildes than any inner element's body
/// delimiter, so an outer scan for `(level + 1)` tildes never collides
/// with any nested level-`k` (k < level) delimiter.
pub fn compute_body_level(orders: &[u32]) -> u32 {
    orders.iter().copied().max().map(|m| m + 1).unwrap_or(1)
}

/// Wrap one input's full file content in level-`body_level` open/close
/// delimiters. Each delimiter sits on its own line.
fn wrap_chunk(body_level: u32, full_file_content: &str) -> String {
    let open = body_delimiter_open(body_level);
    let close = body_delimiter_close(body_level);
    let trimmed = full_file_content.trim_end_matches('\n');
    format!("{open}\n{trimmed}\n{close}")
}

/// Render the body of a composed element by concatenating the full-file
/// content of each input separated by level-`body_level` delimiters.
pub fn render_body(body_level: u32, input_files: &[String]) -> String {
    input_files
        .iter()
        .map(|f| wrap_chunk(body_level, f))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Resolve and render a Compose request into a [`PromptElement`].
pub fn compose(req: ComposeRequest<'_>) -> Result<PromptElement> {
    if req.inputs.is_empty() {
        return Err(OovraError::EmptyCompose);
    }

    // Resolve inputs against the library, enforcing version pins.
    let mut resolved: Vec<&PromptElement> = Vec::with_capacity(req.inputs.len());
    let mut input_refs: Vec<InputRef> = Vec::with_capacity(req.inputs.len());

    for (id, pin) in &req.inputs {
        let element = req
            .library
            .get(id)
            .ok_or_else(|| OovraError::ElementNotFound { id: id.clone() })?;

        if let Some(pin) = pin {
            if &element.header.version != pin {
                return Err(OovraError::VersionMismatch {
                    id: id.clone(),
                    pin: pin.clone(),
                    actual: element.header.version.clone(),
                });
            }
        }

        resolved.push(element);
        input_refs.push(InputRef::new(id.clone(), element.header.version.clone()));
    }

    // Compute logical order and physical body delimiter level. They are
    // distinct: the user's order formula does not always climb, but the
    // delimiter level always escalates to satisfy strict monotonicity.
    let input_orders: Vec<u32> = resolved.iter().map(|e| e.header.order).collect();
    let output_order = compute_order(&input_orders);
    let body_level = compute_body_level(&input_orders);

    // Render each input as a complete file string (frontmatter + body),
    // wrap each in level-`body_level` delimiters, and concatenate.
    let mut input_files: Vec<String> = Vec::with_capacity(resolved.len());
    for input in &resolved {
        input_files.push(serialize(input)?);
    }
    let body = render_body(body_level, &input_files);

    let header = PromptElementHeader {
        name: req.output_name,
        kind: PromptElementKind::Compound,
        order: output_order,
        id: req.output_id,
        version: req.output_version,
        meta: req.output_meta,
        generated_at: Some(Utc::now().to_rfc3339()),
        render_mode: Some("markdown-h2".to_string()),
        body_level: Some(body_level),
        depth: None,
        composed_of: Some(input_refs),
    };

    Ok(PromptElement::new(header, body))
}

/// Render a clean human-readable prompt from a list of inputs, suitable for
/// the `compose --text` flag. Each input's body is wrapped in a Markdown H2
/// containing its ID; sub-element headers are stripped.
///
/// This is intentionally NOT a valid Oovra file — it's the "give me a prompt
/// to paste into a model" output. Use `compose` (without `--text`) when you
/// want a self-describing on-disk artifact.
pub fn render_text(inputs: &[&PromptElement]) -> Result<String> {
    let parts: Vec<String> = inputs
        .iter()
        .map(|e| render_for_paste(e))
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}

/// Recursively render a prompt element to its prose form: an H2 header per
/// "leaf" (order-0) element, with no Oovra metadata leaking into the output.
/// For order >= 1 elements, the body is parsed for embedded sub-elements and
/// the recursion descends.
fn render_for_paste(element: &PromptElement) -> Result<String> {
    if element.header.is_atomic() {
        return Ok(format!("## {}\n\n{}", element.header.id, element.body.trim()));
    }
    // Composed element: split its body into immediate sub-elements and
    // render each. This collapses arbitrarily nested compositions into a
    // flat list of H2-wrapped order-0 bodies.
    let sub_elements = crate::decompose::decompose(element)?;
    let parts: Vec<String> = sub_elements
        .iter()
        .map(render_for_paste)
        .collect::<Result<Vec<_>>>()?;
    Ok(parts.join("\n\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_order_examples_from_spec() {
        // 3 order-0 inputs -> order 1
        assert_eq!(compute_order(&[0, 0, 0]), 1);
        // 2 order-1 inputs -> order 2
        assert_eq!(compute_order(&[1, 1]), 2);
        // 1 order-1 + 3 order-0 -> stays at 1
        assert_eq!(compute_order(&[1, 0, 0, 0]), 1);
        // 1 order-2 + 1 order-1 + 5 order-0 -> stays at 2
        assert_eq!(compute_order(&[2, 1, 0, 0, 0, 0, 0]), 2);
        // 2 order-2 + 1 order-1 -> order 3
        assert_eq!(compute_order(&[2, 2, 1]), 3);
        // 1 input is degenerate identity
        assert_eq!(compute_order(&[0]), 0);
        assert_eq!(compute_order(&[3]), 3);
        // Empty fallback
        assert_eq!(compute_order(&[]), 0);
    }

    #[test]
    fn render_body_wraps_each_input_in_correct_delimiters() {
        let chunks = vec!["FILE_A".to_string(), "FILE_B".to_string()];
        let body = render_body(1, &chunks);
        assert!(body.contains("~~>>\nFILE_A\n~~<<"));
        assert!(body.contains("~~>>\nFILE_B\n~~<<"));

        let body2 = render_body(2, &chunks);
        assert!(body2.contains("~~~>>\nFILE_A\n~~~<<"));
        assert!(body2.contains("~~~>>\nFILE_B\n~~~<<"));
    }

    #[test]
    fn body_level_always_strictly_greater_than_max_input_order() {
        assert_eq!(compute_body_level(&[0, 0, 0]), 1);
        assert_eq!(compute_body_level(&[1, 1]), 2);
        assert_eq!(compute_body_level(&[1, 0, 0, 0]), 2);
        assert_eq!(compute_body_level(&[2, 1, 0, 0, 0]), 3);
        assert_eq!(compute_body_level(&[2, 2, 1]), 3);
        assert_eq!(compute_body_level(&[5]), 6);
        assert_eq!(compute_body_level(&[]), 1);
    }

    #[test]
    fn order_and_body_level_diverge_when_count_at_max_is_one() {
        // count_at_max == 1 — order does not climb, but body_level always does.
        assert_eq!(compute_order(&[1, 0]), 1);
        assert_eq!(compute_body_level(&[1, 0]), 2);

        assert_eq!(compute_order(&[2, 1, 0, 0]), 2);
        assert_eq!(compute_body_level(&[2, 1, 0, 0]), 3);

        // Single input edge case.
        assert_eq!(compute_order(&[3]), 3);
        assert_eq!(compute_body_level(&[3]), 4);
    }
}
