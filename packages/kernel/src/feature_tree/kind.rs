/// Enumeration of all feature types supported by the kernel.
/// Each variant corresponds to an Operation implementation.
///
/// All variants are always present for deserialization compatibility —
/// the server feature flag only gates the Operation dispatch, not the enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FeatureKind {
    // Sketch
    Sketch,

    // Client operations
    Extrude,
    Revolve,
    Fillet,
    Chamfer,

    // Server-only operations (enum variants always present for deserialization)
    BooleanUnion,
    BooleanSubtract,
    BooleanIntersect,
    Sweep,
    Loft,
    Shell,
    Draft,
    LinearPattern,
    CircularPattern,
    Mirror,
}

impl FeatureKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            FeatureKind::Sketch => "Sketch",
            FeatureKind::Extrude => "Extrude",
            FeatureKind::Revolve => "Revolve",
            FeatureKind::Fillet => "Fillet",
            FeatureKind::Chamfer => "Chamfer",
            FeatureKind::BooleanUnion => "Boolean Union",
            FeatureKind::BooleanSubtract => "Boolean Subtract",
            FeatureKind::BooleanIntersect => "Boolean Intersect",
            FeatureKind::Sweep => "Sweep",
            FeatureKind::Loft => "Loft",
            FeatureKind::Shell => "Shell",
            FeatureKind::Draft => "Draft",
            FeatureKind::LinearPattern => "Linear Pattern",
            FeatureKind::CircularPattern => "Circular Pattern",
            FeatureKind::Mirror => "Mirror",
        }
    }

    /// Whether this operation requires the server feature
    pub fn requires_server(&self) -> bool {
        matches!(
            self,
            FeatureKind::BooleanUnion
                | FeatureKind::BooleanSubtract
                | FeatureKind::BooleanIntersect
                | FeatureKind::Sweep
                | FeatureKind::Loft
                | FeatureKind::Shell
                | FeatureKind::Draft
                | FeatureKind::LinearPattern
                | FeatureKind::CircularPattern
                | FeatureKind::Mirror
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_serializes_as_snake_case() {
        let json = serde_json::to_string(&FeatureKind::BooleanUnion).unwrap();
        assert_eq!(json, r#""boolean_union""#);
    }

    #[test]
    fn kind_deserializes_from_snake_case() {
        let kind: FeatureKind = serde_json::from_str(r#""linear_pattern""#).unwrap();
        assert_eq!(kind, FeatureKind::LinearPattern);
    }

    #[test]
    fn client_ops_dont_require_server() {
        assert!(!FeatureKind::Extrude.requires_server());
        assert!(!FeatureKind::Revolve.requires_server());
        assert!(!FeatureKind::Fillet.requires_server());
        assert!(!FeatureKind::Chamfer.requires_server());
    }

    #[test]
    fn server_ops_require_server() {
        assert!(FeatureKind::BooleanUnion.requires_server());
        assert!(FeatureKind::Sweep.requires_server());
        assert!(FeatureKind::Loft.requires_server());
    }
}
