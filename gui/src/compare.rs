//! Compare: the GUI's third central-panel tab. Pick two elements
//! from the loaded library, render `oovra::diff::compare`'s output.
//!
//! Sprint s4. No new library code — `oovra::diff::compare` already
//! produces a `DiffReport` (atom-vs-atom Content / compound-vs-
//! compound Structural / mixed-kind error). The GUI re-renders the
//! same data with egui widgets.

use oovra::Library;
use oovra::diff::{DiffReport, compare};

/// Working state for the Compare tab. Lives on `OovraApp`; not
/// persisted across runs.
#[derive(Debug, Default)]
pub struct CompareState {
    /// Id of the element on the "before" side.
    pub a: Option<String>,
    /// Id of the element on the "after" side.
    pub b: Option<String>,
    /// Cached compare result. `None` if either side isn't picked
    /// or both sides are the same id (no work to compare).
    /// `Some(Ok(_))` is a real report; `Some(Err(_))` is the
    /// mixed-kind / missing-id error path.
    pub report: Option<Result<DiffReport, String>>,
}

impl CompareState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replace the A pick and recompute against `lib`.
    pub fn set_a(&mut self, id: Option<String>, lib: Option<&Library>) {
        self.a = id;
        self.recompute(lib);
    }

    /// Replace the B pick and recompute against `lib`.
    pub fn set_b(&mut self, id: Option<String>, lib: Option<&Library>) {
        self.b = id;
        self.recompute(lib);
    }

    /// Recompute the diff against the supplied library. Public so
    /// the host can call it when the loaded library changes
    /// underneath (e.g. on save-as-compound).
    pub fn recompute(&mut self, lib: Option<&Library>) {
        let (Some(a), Some(b), Some(lib)) = (self.a.as_ref(), self.b.as_ref(), lib) else {
            self.report = None;
            return;
        };
        if a == b {
            self.report = None;
            return;
        }
        let Some(ea) = lib.get(a) else {
            self.report = Some(Err(format!("'{a}' is not in the loaded olib")));
            return;
        };
        let Some(eb) = lib.get(b) else {
            self.report = Some(Err(format!("'{b}' is not in the loaded olib")));
            return;
        };
        self.report = Some(compare(ea, eb).map_err(|e| e.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oovra::render::{ComposeRequest, compose};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn tempdir(name: &str) -> PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir =
            std::env::temp_dir().join(format!("oovra-compare-{}-{}-{n}", name, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// Build an olib with two atoms (differ in body) and one compound,
    /// then return the loaded Library.
    fn fixture(dir: &Path) -> Library {
        let olib = dir.join("olib");
        fs::create_dir_all(&olib).unwrap();
        oovra::create::label_into_olib(&olib, "role body v1", "role", "1.0.0", "the role").unwrap();
        oovra::create::label_into_olib(&olib, "role body v2", "role-alt", "1.0.0", "alt role")
            .unwrap();
        // Build a compound so the mixed-kind test has a compound to point at.
        let lib_for_compose = Library::load_with(&olib, oovra::ParseOptions::default()).unwrap();
        let composed = compose(ComposeRequest {
            library: &lib_for_compose,
            inputs: vec![("role".to_owned(), Some("1.0.0".to_owned()))],
            output_id: "role-prompt".to_owned(),
            output_name: "role-prompt".to_owned(),
            output_version: "1.0.0".to_owned(),
            output_meta: String::new(),
        })
        .unwrap();
        oovra::write(&composed, &olib.join("role-prompt.md")).unwrap();
        Library::load_with(&olib, oovra::ParseOptions::default()).unwrap()
    }

    #[test]
    fn compare_state_clears_report_when_either_side_unset() {
        // U4-6: empty picks -> no report.
        let dir = tempdir("empty");
        let lib = fixture(&dir);

        let mut c = CompareState::new();
        c.set_a(Some("role".to_owned()), Some(&lib));
        assert!(c.report.is_none(), "one side set is not enough");

        c.set_b(Some("role-alt".to_owned()), Some(&lib));
        assert!(c.report.is_some(), "both sides set should produce a report");

        c.set_a(None, Some(&lib));
        assert!(c.report.is_none(), "clearing A clears the report");
    }

    #[test]
    fn compare_state_produces_content_diff_for_two_atoms() {
        // U4-7: atom-vs-atom -> Content diff with bodies_equal == false.
        let dir = tempdir("content");
        let lib = fixture(&dir);

        let mut c = CompareState::new();
        c.set_a(Some("role".to_owned()), Some(&lib));
        c.set_b(Some("role-alt".to_owned()), Some(&lib));

        let report = c
            .report
            .as_ref()
            .expect("expected a report")
            .as_ref()
            .expect("expected Ok");
        match report {
            DiffReport::Content(c) => {
                assert!(!c.bodies_equal);
                assert!(!c.body_unified_diff.is_empty());
            }
            DiffReport::Structural(_) => panic!("expected Content variant for atom-vs-atom"),
        }
    }

    #[test]
    fn compare_state_errors_on_mixed_kind() {
        // U4-8: atom vs compound -> Err (KindMismatch from oovra::diff).
        let dir = tempdir("mixed");
        let lib = fixture(&dir);

        let mut c = CompareState::new();
        c.set_a(Some("role".to_owned()), Some(&lib));
        c.set_b(Some("role-prompt".to_owned()), Some(&lib));

        let r = c.report.as_ref().expect("expected a report");
        assert!(r.is_err(), "atom vs compound must error");
    }

    #[test]
    fn compare_state_same_id_both_sides_clears() {
        // Picking the same id on A and B isn't a comparison.
        let dir = tempdir("same");
        let lib = fixture(&dir);

        let mut c = CompareState::new();
        c.set_a(Some("role".to_owned()), Some(&lib));
        c.set_b(Some("role".to_owned()), Some(&lib));
        assert!(c.report.is_none(), "same id on both sides clears report");
    }
}
