use crate::types::{namespace, Finding};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::{AtomicBool, Ordering},
};

pub fn resource_id(value: &str) -> Option<(String, String)> {
    let (ns, path) = value.split_once(':').unwrap_or(("minecraft", value));
    let valid_ns = !ns.is_empty()
        && ns
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_.-".contains(c));
    let valid_path = !path.is_empty()
        && path
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || "_./-".contains(c))
        && path
            .split('/')
            .all(|s| !s.is_empty() && s != "." && s != "..");
    (valid_ns && valid_path).then(|| (ns.into(), path.into()))
}

fn reference(
    value: &str,
    kind: &str,
    file: &str,
    files: &BTreeMap<String, Vec<u8>>,
    findings: &mut Vec<Finding>,
) {
    let Some((ns, id)) = resource_id(value) else {
        findings.push(Finding::new(
            "error",
            "References",
            "reference.identifier",
            file,
            format!("Invalid resource identifier '{value}'. Use lowercase namespace:path."),
        ));
        return;
    };
    if kind == "models"
        && ns == "minecraft"
        && (id.starts_with("builtin/")
            || ["item/generated", "item/handheld", "item/handheld_rod"].contains(&id.as_str()))
    {
        return;
    }
    let extension = match kind {
        "models" => "json",
        "textures" => "png",
        _ => "ogg",
    };
    let target = format!("assets/{ns}/{kind}/{id}.{extension}");
    if files.contains_key(&target) {
        return;
    }
    let external = ns == "minecraft" || namespace(file).as_deref() != Some(&ns);
    let mut finding = Finding::new(
        if external { "info" } else { "warning" },
        "References",
        if external {
            "reference.external"
        } else {
            "reference.missing"
        },
        file,
        if external {
            format!("External {kind} reference '{value}' is not included. It may be provided by vanilla or another pack/mod.")
        } else {
            format!(
                "Missing local {kind} resource '{value}'. Check the path or supply the dependency."
            )
        },
    );
    finding.context = Some(target);
    findings.push(finding);
}

fn model_references(
    value: &Value,
    file: &str,
    files: &BTreeMap<String, Vec<u8>>,
    findings: &mut Vec<Finding>,
) {
    match value {
        Value::Object(object) => {
            for (key, item) in object {
                if key == "model" {
                    if let Some(id) = item.as_str() {
                        reference(id, "models", file, files, findings);
                    }
                } else {
                    model_references(item, file, files, findings);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                model_references(item, file, files, findings);
            }
        }
        _ => {}
    }
}

fn inherited_textures(
    file: &str,
    documents: &BTreeMap<String, Value>,
    visited: &mut BTreeSet<String>,
    textures: &mut BTreeMap<String, String>,
) -> bool {
    if visited.len() >= 128 || !visited.insert(file.into()) {
        return false;
    }
    let Some(doc) = documents.get(file) else {
        return false;
    };
    let mut complete = true;
    if let Some(parent) = doc.get("parent").and_then(Value::as_str) {
        if let Some((ns, id)) = resource_id(parent) {
            let target = format!("assets/{ns}/models/{id}.json");
            complete = inherited_textures(&target, documents, visited, textures);
        }
    }
    if let Some(map) = doc.get("textures").and_then(Value::as_object) {
        for (key, value) in map {
            if let Some(value) = value.as_str() {
                textures.insert(key.clone(), value.into());
            }
        }
    }
    complete
}

fn aliases(value: &Value, result: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if key == "texture" {
                    if let Some(s) = value.as_str().and_then(|s| s.strip_prefix('#')) {
                        result.insert(s.into());
                    }
                } else {
                    aliases(value, result);
                }
            }
        }
        Value::Array(items) => {
            for value in items {
                aliases(value, result);
            }
        }
        _ => {}
    }
}

pub fn validate(
    documents: &BTreeMap<String, Value>,
    files: &BTreeMap<String, Vec<u8>>,
    findings: &mut Vec<Finding>,
    cancel: &AtomicBool,
) -> Result<(), String> {
    for (file, doc) in documents {
        if cancel.load(Ordering::Relaxed) {
            return Err("Scan cancelled.".into());
        }
        if file == "pack.mcmeta" {
            if doc.get("pack").and_then(Value::as_object).is_none() {
                findings.push(Finding::new(
                    "warning",
                    "Metadata",
                    "pack.structure",
                    file,
                    "Expected a pack object. Confirm this is resource pack metadata.",
                ));
            } else if doc.pointer("/pack/description").is_none() {
                findings.push(Finding::new(
                    "warning",
                    "Metadata",
                    "pack.description",
                    file,
                    "The pack description is absent.",
                ));
            }
        }
        if file.contains("/models/") {
            if !doc.is_object() {
                findings.push(Finding::new(
                    "warning",
                    "Models",
                    "model.structure",
                    file,
                    "Expected a model object. Custom loader formats may differ.",
                ));
                continue;
            }
            if let Some(parent) = doc.get("parent").and_then(Value::as_str) {
                reference(parent, "models", file, files, findings);
            }
            if let Some(map) = doc.get("textures").and_then(Value::as_object) {
                for value in map
                    .values()
                    .filter_map(Value::as_str)
                    .filter(|s| !s.starts_with('#'))
                {
                    reference(value, "textures", file, files, findings);
                }
            }
            if let Some(overrides) = doc.get("overrides") {
                model_references(overrides, file, files, findings);
            }
            let mut textures = BTreeMap::new();
            let complete = inherited_textures(file, documents, &mut BTreeSet::new(), &mut textures);
            let mut used = BTreeSet::new();
            aliases(doc, &mut used);
            used.extend(
                textures
                    .values()
                    .filter_map(|s| s.strip_prefix('#').map(String::from)),
            );
            for alias in used {
                let mut key = alias.clone();
                let mut visited = BTreeSet::new();
                loop {
                    if !visited.insert(key.clone()) {
                        findings.push(Finding::new(
                            "warning",
                            "References",
                            "texture.alias-cycle",
                            file,
                            format!("Texture variable '#{alias}' contains an alias cycle."),
                        ));
                        break;
                    }
                    match textures.get(&key) {
                        Some(value) if value.starts_with('#') => {
                            key = value[1..].into();
                        }
                        Some(_) => break,
                        None => {
                            if complete && !textures.is_empty() {
                                findings.push(Finding::new("warning", "References", "texture.alias", file, format!("Texture variable '#{alias}' has no definition in the local inheritance chain.")));
                            }
                            break;
                        }
                    }
                }
            }
            let mut current = file.clone();
            let mut chain = BTreeSet::new();
            for _ in 0..128 {
                if !chain.insert(current.clone()) {
                    findings.push(Finding::new(
                        "warning",
                        "Models",
                        "model.parent-cycle",
                        file,
                        "Model parent inheritance contains a cycle.",
                    ));
                    break;
                }
                let Some(parent) = documents
                    .get(&current)
                    .and_then(|d| d.get("parent"))
                    .and_then(Value::as_str)
                    .and_then(resource_id)
                else {
                    break;
                };
                current = format!("assets/{}/models/{}.json", parent.0, parent.1);
            }
        }
        if file.contains("/blockstates/") {
            if doc.get("variants").is_none() && doc.get("multipart").is_none() {
                findings.push(Finding::new(
                    "warning",
                    "Blockstates",
                    "blockstate.structure",
                    file,
                    "Expected variants or multipart. Check for a custom format.",
                ));
            }
            model_references(doc, file, files, findings);
        }
        if file.ends_with("/sounds.json") {
            if let Some(events) = doc.as_object() {
                for event in events.values() {
                    if let Some(sounds) = event.get("sounds").and_then(Value::as_array) {
                        for sound in sounds {
                            if sound.get("type").and_then(Value::as_str) == Some("event") {
                                continue;
                            }
                            if let Some(id) = sound
                                .as_str()
                                .or_else(|| sound.get("name").and_then(Value::as_str))
                            {
                                reference(id, "sounds", file, files, findings);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}
