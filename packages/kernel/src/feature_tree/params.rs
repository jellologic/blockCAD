/// Union of all operation parameter types.
/// Serializable for persistence and undo/redo.
///
/// Uses adjacently-tagged representation for clean JSON:
/// `{"type": "extrude", "params": {"direction": [0,0,1], "depth": 10}}`
///
/// Server-only variants are always present in the enum for deserialization,
/// but the operation implementations are gated behind `#[cfg(feature = "server")]`.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "params", rename_all = "snake_case")]
pub enum FeatureParams {
    /// Placeholder for stubs during development
    Placeholder,

    // Client operations
    Sketch(crate::sketch::Sketch),
    Extrude(crate::operations::extrude::ExtrudeParams),
    Revolve(crate::operations::revolve::RevolveParams),
    Fillet(crate::operations::fillet::FilletParams),
    Chamfer(crate::operations::chamfer::ChamferParams),

    // Server-only operations (params always deserializable, execution gated)
    BooleanUnion(serde_json::Value),
    BooleanSubtract(serde_json::Value),
    BooleanIntersect(serde_json::Value),
    Sweep(serde_json::Value),
    Loft(serde_json::Value),
    Shell(serde_json::Value),
    Draft(serde_json::Value),
    LinearPattern(serde_json::Value),
    CircularPattern(serde_json::Value),
    Mirror(serde_json::Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_serializes_cleanly() {
        let p = FeatureParams::Placeholder;
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(json, r#"{"type":"placeholder"}"#);
    }

    #[test]
    fn extrude_params_adjacently_tagged() {
        use crate::geometry::Vec3;
        use crate::operations::extrude::ExtrudeParams;

        let p = FeatureParams::Extrude(ExtrudeParams {
            direction: Vec3::new(0.0, 0.0, 1.0),
            depth: 10.0,
            symmetric: false,
            draft_angle: 0.0,
        });
        let json = serde_json::to_string_pretty(&p).unwrap();
        assert!(json.contains(r#""type": "extrude""#));
        assert!(json.contains(r#""params""#));
        assert!(json.contains(r#""depth": 10.0"#));
    }

    #[test]
    fn server_params_roundtrip_as_value() {
        let json_str = r#"{"type":"sweep","params":{"path_curve_index":0,"twist":0.5}}"#;
        let p: FeatureParams = serde_json::from_str(json_str).unwrap();
        if let FeatureParams::Sweep(v) = &p {
            assert_eq!(v["path_curve_index"], 0);
            assert_eq!(v["twist"], 0.5);
        } else {
            panic!("Expected Sweep variant");
        }
        // Roundtrip
        let json2 = serde_json::to_string(&p).unwrap();
        let p2: FeatureParams = serde_json::from_str(&json2).unwrap();
        assert!(matches!(p2, FeatureParams::Sweep(_)));
    }
}
