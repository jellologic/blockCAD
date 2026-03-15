use crate::feature_tree::Feature;

/// Current schema version. Incremented on breaking changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Schema URL for JSON validation
pub const SCHEMA_URL: &str = "https://blockcad.dev/schema/v1.json";

/// Document metadata for version control and collaboration.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Metadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

impl Metadata {
    pub fn new(name: String) -> Self {
        Self {
            name,
            description: None,
            created_at: None,
            modified_at: None,
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new("Untitled".into())
    }
}

/// The top-level serializable document representing a complete kernel state.
/// Designed for clean git diffs with human-readable JSON.
///
/// File extension: `.blockcad`
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KernelDocument {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
    pub version: u32,
    pub metadata: Metadata,
    pub features: Vec<Feature>,
}

impl KernelDocument {
    pub fn new(name: String, features: Vec<Feature>) -> Self {
        Self {
            schema_url: Some(SCHEMA_URL.into()),
            version: SCHEMA_VERSION,
            metadata: Metadata::new(name),
            features,
        }
    }

    /// Serialize to pretty-printed JSON for git-friendly storage.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_tree::{FeatureKind, FeatureParams};
    use crate::geometry::Vec3;
    use crate::operations::extrude::ExtrudeParams;

    #[test]
    fn document_roundtrip_json() {
        let doc = KernelDocument::new(
            "Test Part".into(),
            vec![Feature::new(
                "extrude-1".into(),
                "Extrude1".into(),
                FeatureKind::Extrude,
                FeatureParams::Placeholder,
            )],
        );
        let json = doc.to_json_pretty().unwrap();
        let doc2 = KernelDocument::from_json(&json).unwrap();
        assert_eq!(doc2.version, SCHEMA_VERSION);
        assert_eq!(doc2.features.len(), 1);
        assert_eq!(doc2.features[0].name, "Extrude1");
        assert_eq!(doc2.metadata.name, "Test Part");
    }

    #[test]
    fn golden_json_format() {
        let doc = KernelDocument::new(
            "My Part".into(),
            vec![
                Feature::new(
                    "extrude-1".into(),
                    "Extrude Base".into(),
                    FeatureKind::Extrude,
                    FeatureParams::Extrude(ExtrudeParams {
                        direction: Vec3::new(0.0, 0.0, 1.0),
                        depth: 10.0,
                        symmetric: false,
                        draft_angle: 0.0,
                    }),
                ),
                Feature::new(
                    "fillet-1".into(),
                    "Edge Fillet".into(),
                    FeatureKind::Fillet,
                    FeatureParams::Fillet(crate::operations::fillet::FilletParams {
                        edge_indices: vec![0, 2, 5],
                        radius: 1.5,
                    }),
                ),
            ],
        );
        let json = doc.to_json_pretty().unwrap();

        // Verify structure
        assert!(json.contains(r#""$schema": "https://blockcad.dev/schema/v1.json""#));
        assert!(json.contains(r#""version": 1"#));
        assert!(json.contains(r#""name": "My Part""#));

        // Features use adjacently-tagged format with snake_case
        assert!(json.contains(r#""type": "extrude""#));
        assert!(json.contains(r#""depth": 10.0"#));
        assert!(json.contains(r#""type": "fillet""#));
        assert!(json.contains(r#""radius": 1.5"#));

        // Feature fields present
        assert!(json.contains(r#""id": "extrude-1""#));
        assert!(json.contains(r#""suppressed": false"#));

        // No transient state
        assert!(!json.contains("Pending"));
        assert!(!json.contains("Evaluated"));
    }

    #[test]
    fn server_only_features_deserialize_on_client() {
        // A document with server-only features should still parse on client
        let json = r#"{
            "$schema": "https://blockcad.dev/schema/v1.json",
            "version": 1,
            "metadata": { "name": "Test" },
            "features": [
                {
                    "id": "sweep-1",
                    "name": "Sweep Path",
                    "type": "sweep",
                    "suppressed": false,
                    "params": { "type": "sweep", "params": {"path_curve_index": 0, "twist": 0.5} }
                }
            ]
        }"#;
        let doc = KernelDocument::from_json(json).unwrap();
        assert_eq!(doc.features.len(), 1);
        assert_eq!(doc.features[0].kind, FeatureKind::Sweep);
    }
}
