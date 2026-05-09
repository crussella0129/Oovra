//! End-to-end pipeline tests: parse, compose order-1 from order-0s, compose
//! order-2 from order-1s, decompose one level, decompose --full, compare.

use std::path::Path;

use oovra::decompose::{decompose, decompose_full};
use oovra::diff::{compare, DiffReport};
use oovra::element::{parse_file, write};
use oovra::library::Library;
use oovra::render::{compose, ComposeRequest};

/// Path to the sample library shipped in this repo.
fn elements_dir() -> &'static Path {
    Path::new("elements")
}

#[test]
fn create_with_invalid_id_does_not_leave_orphan_file() {
    // Regression: Create used to fs::write before validating, so a bad id
    // left an unparseable file on disk. After the in-memory pre-validation
    // fix, no file should be written.
    use oovra::create::{scaffold, ScaffoldArgs};
    let tmp = tempdir_for_test("orphan-check");
    let result = scaffold(ScaffoldArgs {
        library_dir: tmp.clone(),
        id: "BadID".into(),  // not kebab-case
        name: None,
        version: "1.0.0".into(),
        meta: String::new(),
    });
    assert!(result.is_err(), "expected scaffold to reject 'BadID'");
    let entries: Vec<_> = std::fs::read_dir(&tmp).unwrap().filter_map(|e| e.ok()).collect();
    assert!(entries.is_empty(), "expected no files written, found {:?}", entries.iter().map(|e| e.path()).collect::<Vec<_>>());
}

#[test]
fn parse_serialize_round_trip_is_idempotent() {
    // Load each shipped order-0 element, serialize, parse, serialize again,
    // and assert the second serialization equals the first. This catches
    // serializer non-determinism and parse-write drift.
    let library = Library::load(elements_dir()).unwrap();
    for element in library.elements.values() {
        let s1 = oovra::element::serialize(element).unwrap();
        let parsed = oovra::element::parse(&s1, std::path::Path::new("<test>")).unwrap();
        let s2 = oovra::element::serialize(&parsed).unwrap();
        assert_eq!(s1, s2, "non-idempotent round-trip for '{}'", element.header.id);
    }
}

#[test]
fn compose_round_trip_preserves_recipe_and_decomposes_lossless() {
    // Compose -> serialize -> parse -> decompose -> compare each leaf to
    // original. Catches any byte drift that would break decompose.
    let library = Library::load(elements_dir()).unwrap();
    let composed = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "round-trip".into(),
        output_name: "Round Trip Test".into(),
        output_version: "1.0.0".into(),
        output_meta: "ensures byte-stable round-trip".into(),
    })
    .unwrap();

    let serialized = oovra::element::serialize(&composed).unwrap();
    let reparsed = oovra::element::parse(&serialized, std::path::Path::new("<rt>")).unwrap();

    // Header round-trips exactly.
    assert_eq!(reparsed.header.id, composed.header.id);
    assert_eq!(reparsed.header.order, composed.header.order);
    assert_eq!(reparsed.header.body_level, composed.header.body_level);
    assert_eq!(reparsed.header.composed_of, composed.header.composed_of);

    // Decompose recovers the original inputs byte-for-byte.
    let recovered = decompose(&reparsed).unwrap();
    for original_id in &["role-declaration", "refusal-policy-strict", "tone-direct"] {
        let original = library.get(original_id).unwrap();
        let rec = recovered.iter().find(|e| e.header.id == *original_id).unwrap();
        assert_eq!(rec.body, original.body, "body drift for {original_id}");
        assert_eq!(rec.header.name, original.header.name);
        assert_eq!(rec.header.version, original.header.version);
        assert_eq!(rec.header.meta, original.header.meta);
    }
}

#[test]
fn library_loads_five_order_zero_elements() {
    let library = Library::load(elements_dir()).unwrap();
    assert_eq!(library.len(), 5);
    for element in library.elements.values() {
        assert_eq!(element.header.order, 0, "{}", element.header.id);
    }
}

#[test]
fn compose_three_order_zero_into_one_order_one() {
    let library = Library::load(elements_dir()).unwrap();
    let req = ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "coding-agent-strict".into(),
        output_name: "Coding Agent (Strict)".into(),
        output_version: "1.0.0".into(),
        output_meta: "Three-element strict coding-agent prompt".into(),
    };
    let composed = compose(req).unwrap();
    assert_eq!(composed.header.order, 1);
    assert_eq!(
        composed.header.composed_of.as_ref().map(|v| v.len()),
        Some(3)
    );
    // Each input's content should appear in the body.
    for id in &[
        "role-declaration",
        "refusal-policy-strict",
        "tone-direct",
    ] {
        assert!(composed.body.contains(id), "expected {id} to appear in body");
    }
    // Body should contain the level-1 delimiters.
    assert!(composed.body.contains("~~>>"));
    assert!(composed.body.contains("~~<<"));
}

#[test]
fn compose_two_order_one_into_one_order_two() {
    let library = Library::load(elements_dir()).unwrap();

    // Build two distinct order-1 elements.
    let sub_a = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "subprompt-a".into(),
        output_name: "Subprompt A".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let sub_b = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("output-format-markdown".into(), None),
            ("examples-block".into(), None),
        ],
        output_id: "subprompt-b".into(),
        output_name: "Subprompt B".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage them into a temp directory so we can re-compose from the library
    // side. (Compose resolves inputs via Library, so they must be on disk.)
    let tmp = tempdir_for_test("order-2-staging");
    write(&sub_a, &tmp.join("subprompt-a.md")).unwrap();
    write(&sub_b, &tmp.join("subprompt-b.md")).unwrap();

    // Copy the order-0 elements into the same staging dir so they are
    // resolvable too (for the body's nested decomposition later).
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }

    let staged_lib = Library::load(&tmp).unwrap();
    let order_two = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![
            ("subprompt-a".into(), None),
            ("subprompt-b".into(), None),
        ],
        output_id: "two-stage-prompt".into(),
        output_name: "Two-Stage Prompt".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    assert_eq!(order_two.header.order, 2);
    assert!(order_two.body.contains("~~~>>"));
    assert!(order_two.body.contains("~~~<<"));
    // Inner level-1 delimiters must be preserved verbatim.
    assert!(order_two.body.contains("~~>>"));
    assert!(order_two.body.contains("~~<<"));
}

#[test]
fn decompose_one_level_recovers_immediate_inputs() {
    let library = Library::load(elements_dir()).unwrap();
    let composed = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "tmp-compose".into(),
        output_name: "tmp".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    let inputs = decompose(&composed).unwrap();
    assert_eq!(inputs.len(), 3);
    assert_eq!(inputs[0].header.id, "role-declaration");
    assert_eq!(inputs[1].header.id, "refusal-policy-strict");
    assert_eq!(inputs[2].header.id, "tone-direct");

    // Every recovered input must round-trip exactly.
    let original_a = library.get("role-declaration").unwrap();
    assert_eq!(inputs[0].body, original_a.body);
    assert_eq!(inputs[0].header.version, original_a.header.version);
}

#[test]
fn decompose_full_writes_folder_tree_for_order_two() {
    let tmp = tempdir_for_test("decompose-full");
    let library = Library::load(elements_dir()).unwrap();

    // Build two order-1 sub-prompts.
    let sub_a = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "subprompt-a".into(),
        output_name: "Subprompt A".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let sub_b = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("output-format-markdown".into(), None),
            ("tone-direct".into(), None),
        ],
        output_id: "subprompt-b".into(),
        output_name: "Subprompt B".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage to a temp library that contains the order-0s and the two order-1s
    let staging = tmp.join("staging");
    std::fs::create_dir_all(&staging).unwrap();
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, staging.join(p.file_name().unwrap())).unwrap();
    }
    write(&sub_a, &staging.join("subprompt-a.md")).unwrap();
    write(&sub_b, &staging.join("subprompt-b.md")).unwrap();

    let staged_lib = Library::load(&staging).unwrap();
    let order_two = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![
            ("subprompt-a".into(), None),
            ("subprompt-b".into(), None),
        ],
        output_id: "full-test".into(),
        output_name: "Full Test".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    let out_dir = tmp.join("out");
    let element_root = decompose_full(&order_two, &out_dir).unwrap();
    assert!(element_root.is_dir(), "{} should be a directory", element_root.display());

    // Expected structure:
    //   out/full-test/
    //     full-test.md
    //     subprompt-a/
    //       subprompt-a.md
    //       role-declaration.md
    //       refusal-policy-strict.md
    //     subprompt-b/
    //       subprompt-b.md
    //       output-format-markdown.md
    //       tone-direct.md

    assert!(element_root.join("full-test.md").is_file());
    assert!(element_root.join("subprompt-a").is_dir());
    assert!(element_root.join("subprompt-a/subprompt-a.md").is_file());
    assert!(element_root.join("subprompt-a/role-declaration.md").is_file());
    assert!(element_root.join("subprompt-a/refusal-policy-strict.md").is_file());
    assert!(element_root.join("subprompt-b").is_dir());
    assert!(element_root.join("subprompt-b/subprompt-b.md").is_file());
    assert!(element_root.join("subprompt-b/output-format-markdown.md").is_file());
    assert!(element_root.join("subprompt-b/tone-direct.md").is_file());

    // Each leaf must round-trip parse cleanly with original metadata.
    let leaf = parse_file(&element_root.join("subprompt-a/role-declaration.md")).unwrap();
    let original = library.get("role-declaration").unwrap();
    assert_eq!(leaf.header.id, original.header.id);
    assert_eq!(leaf.header.version, original.header.version);
    assert_eq!(leaf.header.name, original.header.name);
    assert_eq!(leaf.header.meta, original.header.meta);
    assert_eq!(leaf.body, original.body);
}

#[test]
fn compare_structural_diff_detects_version_change() {
    let library = Library::load(elements_dir()).unwrap();

    let v1 = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage a modified library where refusal-policy-strict is bumped.
    let tmp = tempdir_for_test("structural-diff");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    let bumped_path = tmp.join("refusal-policy-strict.md");
    let mut bumped = parse_file(&bumped_path).unwrap();
    bumped.header.version = "1.1.0".into();
    write(&bumped, &bumped_path).unwrap();

    let bumped_lib = Library::load(&tmp).unwrap();
    let v2 = compose(ComposeRequest {
        library: &bumped_lib,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "stable".into(),
        output_name: "Stable".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    match compare(&v1, &v2).unwrap() {
        DiffReport::Structural(s) => {
            assert!(s.added.is_empty());
            assert!(s.removed.is_empty());
            assert_eq!(s.version_changed.len(), 1);
            assert_eq!(s.version_changed[0].id, "refusal-policy-strict");
            assert_eq!(s.version_changed[0].before_version, "1.0.0");
            assert_eq!(s.version_changed[0].after_version, "1.1.0");
        }
        DiffReport::Content(_) => panic!("expected structural diff for order-1 elements"),
    }
}

#[test]
fn mixed_order_compose_does_not_collide_with_inner_delimiters() {
    // Regression test for the body-delimiter escalation bug: composing an
    // order-1 element with order-0 elements used to produce a body whose
    // top-level delimiters collided with the inner element's body delimiters.
    // After the body_level fix, the top-level delimiter level is always
    // strictly greater than any input's body delimiter level.
    let library = Library::load(elements_dir()).unwrap();

    // Build an order-1 first.
    let inner = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "inner-order-one".into(),
        output_name: "Inner".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Stage so it can be re-resolved as an input.
    let tmp = tempdir_for_test("mixed-order");
    for entry in std::fs::read_dir(elements_dir()).unwrap() {
        let p = entry.unwrap().path();
        std::fs::copy(&p, tmp.join(p.file_name().unwrap())).unwrap();
    }
    write(&inner, &tmp.join("inner-order-one.md")).unwrap();

    let staged_lib = Library::load(&tmp).unwrap();
    let mixed = compose(ComposeRequest {
        library: &staged_lib,
        inputs: vec![
            ("inner-order-one".into(), None),  // order 1
            ("tone-direct".into(), None),      // order 0
        ],
        output_id: "mixed-order".into(),
        output_name: "Mixed".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();

    // Per the user's formula: highest=1, count=1 (no peer at 1), so order
    // does not climb. But body_level MUST climb to keep the parser
    // unambiguous.
    assert_eq!(mixed.header.order, 1);
    assert_eq!(mixed.header.body_level, Some(2));

    // Decompose must succeed because the outer delimiters are level 2
    // (3 tildes) while the inner element's body uses level-1 (2 tildes).
    let immediate = decompose(&mixed).unwrap();
    assert_eq!(immediate.len(), 2);
    assert_eq!(immediate[0].header.id, "inner-order-one");
    assert_eq!(immediate[1].header.id, "tone-direct");

    // The recovered inner is still a valid order-1 with its own body_level=1.
    assert_eq!(immediate[0].header.order, 1);
    assert_eq!(immediate[0].header.body_level, Some(1));
}

#[test]
fn compare_refuses_different_orders() {
    let library = Library::load(elements_dir()).unwrap();
    let order_one = compose(ComposeRequest {
        library: &library,
        inputs: vec![
            ("role-declaration".into(), None),
            ("refusal-policy-strict".into(), None),
        ],
        output_id: "x".into(),
        output_name: "x".into(),
        output_version: "1.0.0".into(),
        output_meta: String::new(),
    })
    .unwrap();
    let order_zero = library.get("role-declaration").unwrap();
    let err = compare(order_zero, &order_one).unwrap_err();
    assert!(matches!(err, oovra::OovraError::OrderMismatch { .. }));
}

/// Lightweight tempdir helper. Creates a unique directory under
/// `target/tmp/<name>-<pid>/` that lives for the duration of the test process.
fn tempdir_for_test(name: &str) -> std::path::PathBuf {
    let dir = std::env::current_dir()
        .unwrap()
        .join("target/tmp")
        .join(format!("{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
