use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub severity: String,
    pub category: String,
    pub code: String,
    pub path: String,
    pub message: String,
    pub context: Option<String>,
    pub line: Option<usize>,
    pub namespace: String,
}

impl Finding {
    pub fn new(
        severity: &str,
        category: &str,
        code: &str,
        path: &str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity: severity.into(),
            category: category.into(),
            code: code.into(),
            path: path.into(),
            message: message.into(),
            context: None,
            line: None,
            namespace: namespace(path).unwrap_or_default(),
        }
    }
}

pub fn namespace(path: &str) -> Option<String> {
    let parts: Vec<_> = path.split('/').collect();
    parts
        .windows(2)
        .find(|p| p[0] == "assets")
        .map(|p| p[1].to_string())
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Texture {
    pub width: u32,
    pub height: u32,
    pub color_mode: String,
    pub has_alpha: bool,
    pub transparent: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    pub path: String,
    pub kind: String,
    pub namespace: String,
    pub bytes: usize,
    pub texture: Option<Texture>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub schema_version: u32,
    pub project_name: String,
    pub source_kind: String,
    pub scanned_at: u64,
    pub duration_ms: u128,
    pub total_bytes: usize,
    pub assets: Vec<Asset>,
    pub findings: Vec<Finding>,
    pub namespaces: Vec<String>,
    pub counts: BTreeMap<String, usize>,
    pub complete: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    pub phase: String,
    pub processed: usize,
    pub total: Option<usize>,
}
