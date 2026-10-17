use crate::{
    source::Source,
    types::{namespace, Asset, Finding, Progress, Report, Texture},
    validation,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
    time::{Instant, SystemTime, UNIX_EPOCH},
};

pub fn inspect_png(bytes: &[u8]) -> Result<Texture, String> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_limits(png::Limits {
        bytes: 64 * 1024 * 1024,
    });
    decoder.set_transformations(png::Transformations::EXPAND);
    let mut reader = decoder
        .read_info()
        .map_err(|e| format!("Invalid PNG header: {e}"))?;
    let original = reader.info();
    let color_mode = format!("{:?} / {:?}", original.color_type, original.bit_depth);
    let has_alpha = matches!(
        original.color_type,
        png::ColorType::Rgba | png::ColorType::GrayscaleAlpha
    ) || original.trns.is_some();
    let size = reader.output_buffer_size();
    if size > 64 * 1024 * 1024 {
        return Err("Decoded texture exceeds the 64 MiB safety limit.".into());
    }
    let mut buffer = vec![0; size];
    let info = reader
        .next_frame(&mut buffer)
        .map_err(|e| format!("Cannot decode PNG: {e}"))?;
    let stride = match info.color_type {
        png::ColorType::Rgba => 4,
        png::ColorType::GrayscaleAlpha => 2,
        _ => 0,
    };
    let sample_bytes = if info.bit_depth == png::BitDepth::Sixteen {
        2
    } else {
        1
    };
    let transparent = stride > 0
        && buffer[..info.buffer_size()]
            .chunks_exact(stride * sample_bytes)
            .any(|pixel| {
                if sample_bytes == 2 {
                    let offset = (stride - 1) * 2;
                    u16::from_be_bytes([pixel[offset], pixel[offset + 1]]) < u16::MAX
                } else {
                    pixel[stride - 1] < u8::MAX
                }
            });
    Ok(Texture {
        width: info.width,
        height: info.height,
        color_mode,
        has_alpha,
        transparent,
    })
}

pub fn kind(path: &str) -> &'static str {
    if path.ends_with(".png") {
        "texture"
    } else if path.contains("/models/") && path.ends_with(".json") {
        "model"
    } else if path.contains("/blockstates/") && path.ends_with(".json") {
        "blockstate"
    } else if path.ends_with(".ogg") || path.ends_with("/sounds.json") {
        "sound"
    } else if path.contains("/lang/") {
        "language"
    } else {
        "metadata"
    }
}

pub fn analyze(
    source: &Source,
    name: String,
    source_kind: String,
    cancel: &AtomicBool,
    progress: &impl Fn(Progress),
) -> Result<Report, String> {
    let start = Instant::now();
    let mut findings = source.findings.clone();
    let mut assets = Vec::new();
    let mut documents = BTreeMap::new();
    for (index, (path, bytes)) in source.files.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err("Scan cancelled.".into());
        }
        let texture = if path.ends_with(".png") {
            match inspect_png(bytes) {
                Ok(texture) => {
                    if !texture.width.is_power_of_two() || !texture.height.is_power_of_two() {
                        findings.push(Finding::new("warning", "Textures", "texture.dimensions", path, format!("Texture is {} × {}. Non-power-of-two dimensions may affect mipmapping; animation strips can be intentional.", texture.width, texture.height)));
                    }
                    Some(texture)
                }
                Err(e) => {
                    findings.push(Finding::new("error", "Textures", "texture.decode", path, e));
                    None
                }
            }
        } else {
            None
        };
        if path.ends_with(".json") || path.ends_with(".mcmeta") {
            match crate::json::parse(bytes) {
                Ok(document) => {
                    if !document.duplicates.is_empty() {
                        findings.push(Finding::new("warning", "JSON", "json.duplicate", path, format!("Duplicate object keys at JSON pointers: {}. Later values override earlier definitions. Up to 100 locations are shown.", document.duplicates.into_iter().collect::<Vec<_>>().join(", "))));
                    }
                    documents.insert(path.clone(), document.value);
                }
                Err(e) => {
                    let mut finding = Finding::new(
                        "error",
                        "JSON",
                        "json.syntax",
                        path,
                        format!("Invalid JSON: {e}"),
                    );
                    finding.line = Some(e.line());
                    findings.push(finding);
                }
            }
        }
        assets.push(Asset {
            path: path.clone(),
            kind: kind(path).into(),
            namespace: namespace(path).unwrap_or_default(),
            bytes: bytes.len(),
            texture,
        });
        if index % 25 == 0 || index + 1 == source.files.len() {
            progress(Progress {
                phase: "Inspecting assets".into(),
                processed: index + 1,
                total: Some(source.files.len()),
            });
        }
    }
    validation::validate(&documents, &source.files, &mut findings, cancel)?;
    let namespaces: Vec<_> = assets
        .iter()
        .map(|a| a.namespace.clone())
        .filter(|s| !s.is_empty())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    if !source.files.contains_key("pack.mcmeta") {
        findings.push(Finding::new("info", "Metadata", "pack.absent", "pack.mcmeta", "No root pack.mcmeta found. This may be a mod asset directory or a pack inside a wrapper folder."));
    }
    findings.sort_by(|a, b| (&a.path, &a.code, &a.message).cmp(&(&b.path, &b.code, &b.message)));
    let counts = ["error", "warning", "info"]
        .into_iter()
        .map(|s| {
            (
                s.to_string(),
                findings.iter().filter(|f| f.severity == s).count(),
            )
        })
        .collect();
    Ok(Report {
        schema_version: 1,
        project_name: name,
        source_kind,
        scanned_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| "System clock precedes the Unix epoch.")?
            .as_secs(),
        duration_ms: start.elapsed().as_millis(),
        total_bytes: assets.iter().map(|a| a.bytes).sum(),
        assets,
        findings,
        namespaces,
        counts,
        complete: source.complete,
    })
}
