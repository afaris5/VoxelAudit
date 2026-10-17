use std::{collections::BTreeMap, io::Write, path::Path, sync::atomic::AtomicBool};
use voxelaudit_lib::{
    engine,
    source::{self, Source},
    validation::resource_id,
};

fn analyze(files: &[(&str, &str)]) -> voxelaudit_lib::types::Report {
    let source = Source {
        files: files
            .iter()
            .map(|(p, v)| (p.to_string(), v.as_bytes().to_vec()))
            .collect(),
        findings: vec![],
        complete: true,
    };
    engine::analyze(
        &source,
        "test".into(),
        "directory".into(),
        &AtomicBool::new(false),
        &|_| {},
    )
    .unwrap()
}

#[test]
fn json_syntax_has_line_and_unknown_formats_are_accepted() {
    let report = analyze(&[
        ("assets/demo/lang/en_us.json", "{\n\"a\": }"),
        ("assets/demo/custom.json", r#"{"unfamiliar":true}"#),
    ]);
    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.severity == "error")
        .collect();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].line, Some(2));
}

#[test]
fn duplicate_keys_preserve_last_values_and_report_nested_locations() {
    let document = voxelaudit_lib::json::parse(
        br#"{"a":1,"a":2,"nested":[{"x/y":false,"x/y":null}],"decimal":1.5,"text":"ok"}"#,
    )
    .unwrap();
    assert_eq!(document.value["a"], 2);
    assert!(document.duplicates.contains("/a"));
    assert!(document.duplicates.contains("/nested/0/x~1y"));
    let report = analyze(&[(
        "assets/demo/lang/en_us.json",
        r#"{"a":"first","a":"second"}"#,
    )]);
    assert_eq!(report.counts["error"], 0);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "json.duplicate" && f.severity == "warning"));
}

#[test]
fn references_distinguish_local_external_and_vanilla() {
    let report = analyze(&[(
        "assets/demo/models/item/tool.json",
        r#"{"parent":"minecraft:item/generated","textures":{"layer0":"demo:item/missing","layer1":"other:item/tool","layer2":"item/vanilla"}}"#,
    )]);
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "reference.missing")
            .count(),
        1
    );
    assert_eq!(
        report
            .findings
            .iter()
            .filter(|f| f.code == "reference.external")
            .count(),
        2
    );
    assert!(!report.findings.iter().any(|f| f.severity == "error"));
}

#[test]
fn namespaces_and_identifier_rules() {
    assert_eq!(
        resource_id("block/stone"),
        Some(("minecraft".into(), "block/stone".into()))
    );
    assert_eq!(
        resource_id("demo:item/tool"),
        Some(("demo".into(), "item/tool".into()))
    );
    for id in [
        "Demo:item/a",
        "demo:../a",
        "demo:a:b",
        ":a",
        "demo:/a",
        "demo:a//b",
    ] {
        assert!(resource_id(id).is_none());
    }
}

#[test]
fn local_parent_and_blockstate_dependencies() {
    let report = analyze(&[
        ("assets/demo/models/block/base.json", "{}"),
        (
            "assets/demo/models/block/child.json",
            r#"{"parent":"demo:block/base"}"#,
        ),
        (
            "assets/demo/blockstates/a.json",
            r#"{"multipart":[{"apply":{"model":"demo:block/missing"}}]}"#,
        ),
    ]);
    let missing: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.code == "reference.missing")
        .collect();
    assert_eq!(missing.len(), 1);
    assert!(missing[0].path.contains("blockstates"));
}

#[test]
fn cyclic_parents_and_aliases_terminate() {
    let report = analyze(&[
        (
            "assets/demo/models/a.json",
            r##"{"parent":"demo:b","textures":{"one":"#two","two":"#one"}}"##,
        ),
        ("assets/demo/models/b.json", r#"{"parent":"demo:a"}"#),
    ]);
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "model.parent-cycle"));
    assert!(report
        .findings
        .iter()
        .any(|f| f.code == "texture.alias-cycle"));
}

#[test]
fn texture_header_alpha_and_corruption() {
    let slate = include_bytes!("../../fixtures/valid-pack/assets/atelier/textures/block/slate.png");
    let texture = engine::inspect_png(slate).unwrap();
    assert_eq!((texture.width, texture.height), (16, 16));
    assert!(!texture.transparent);
    let crystal =
        include_bytes!("../../fixtures/valid-pack/assets/atelier/textures/item/crystal.png");
    assert!(engine::inspect_png(crystal).unwrap().transparent);
    assert!(engine::inspect_png(b"not a png").is_err());
}

#[test]
fn sixteen_bit_alpha_is_not_rounded_to_opaque() {
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Sixteen);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&[0, 0, 0, 0, 0, 0, 255, 254])
            .unwrap();
    }
    let texture = engine::inspect_png(&bytes).unwrap();
    assert!(texture.has_alpha);
    assert!(texture.transparent);
    assert!(texture.color_mode.contains("Sixteen"));
}

#[test]
fn oversized_files_produce_incomplete_imports() {
    let temp = tempfile::tempdir().unwrap();
    let file = std::fs::File::create(temp.path().join("oversized.json")).unwrap();
    file.set_len(source::MAX_FILE + 1).unwrap();
    let source = source::load(temp.path(), &AtomicBool::new(false), &|_| {}).unwrap();
    assert!(!source.complete);
    assert!(source.files.is_empty());
    assert!(source.findings[0].message.contains("Safety limit"));
}

#[test]
fn valid_and_broken_real_fixtures() {
    for (name, expected_errors) in [("valid-pack", 0), ("broken-pack", 1)] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures")
            .join(name);
        let source = source::load(&path, &AtomicBool::new(false), &|_| {}).unwrap();
        let report = engine::analyze(
            &source,
            name.into(),
            "directory".into(),
            &AtomicBool::new(false),
            &|_| {},
        )
        .unwrap();
        assert_eq!(report.counts["error"], expected_errors);
        if name == "broken-pack" {
            assert!(report.counts["warning"] >= 4);
        } else {
            assert_eq!(report.counts["warning"], 0);
        }
    }
}

#[test]
fn archive_paths_are_platform_independent() {
    for path in [
        "../evil.json",
        "/absolute.json",
        "C:/evil.json",
        "assets\\demo\\a.json",
        "assets/../a.json",
        "assets//a.json",
        "./a.json",
        "a\0.json",
    ] {
        assert!(!source::safe_archive_path(path), "{path}");
    }
    assert!(source::safe_archive_path("assets/demo/models/a.json"));
}

#[test]
fn zip_import_rejects_traversal_without_extracting() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("pack.zip");
    let mut zip = zip::ZipWriter::new(std::fs::File::create(&path).unwrap());
    for (name, data) in [
        ("pack.mcmeta", "{}"),
        ("../evil.json", "{}"),
        ("assets/demo/models/a.json", "{}"),
    ] {
        zip.start_file(name, zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(data.as_bytes()).unwrap();
    }
    zip.finish().unwrap();
    let source = source::load(&path, &AtomicBool::new(false), &|_| {}).unwrap();
    assert_eq!(source.files.len(), 2);
    assert!(!source.complete);
    assert!(source.findings[0].message.contains("Unsafe"));
    assert!(!temp.path().join("evil.json").exists());
}

#[test]
fn corrupt_archive_and_nonexistent_project_return_errors() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("bad.zip");
    std::fs::write(&path, b"invalid zip").unwrap();
    assert!(source::load(&path, &AtomicBool::new(false), &|_| {}).is_err());
    assert!(source::load(
        &temp.path().join("absent"),
        &AtomicBool::new(false),
        &|_| {}
    )
    .is_err());
}

#[test]
fn empty_large_and_cancelled_projects() {
    let empty = analyze(&[]);
    assert!(empty.assets.is_empty());
    assert!(empty.complete);
    let source = Source {
        files: (0..2000)
            .map(|i| (format!("assets/demo/lang/{i}.json"), b"{}".to_vec()))
            .collect::<BTreeMap<_, _>>(),
        findings: vec![],
        complete: true,
    };
    let report = engine::analyze(
        &source,
        "many".into(),
        "directory".into(),
        &AtomicBool::new(false),
        &|_| {},
    )
    .unwrap();
    assert_eq!(report.assets.len(), 2000);
    assert!(engine::analyze(
        &source,
        "cancelled".into(),
        "directory".into(),
        &AtomicBool::new(true),
        &|_| {}
    )
    .is_err());
}

#[test]
fn reports_roundtrip_real_results_without_absolute_paths() {
    let report = analyze(&[("assets/demo/lang/en_us.json", "{}")]);
    let bytes = serde_json::to_vec_pretty(&report).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["assets"][0]["bytes"], 2);
    assert_eq!(value["assets"][0]["path"], "assets/demo/lang/en_us.json");
    assert_eq!(value["schemaVersion"], 1);
}
