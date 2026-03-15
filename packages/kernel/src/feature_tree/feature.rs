use crate::id::EntityId;

use super::kind::FeatureKind;
use super::params::FeatureParams;

/// Marker type for feature IDs
#[derive(Debug)]
pub struct FeatureMarker;
pub type FeatureId = EntityId<FeatureMarker>;

/// The evaluation state of a feature (runtime-only, not persisted)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureState {
    Pending,
    Evaluated,
    Failed,
}

/// A single feature in the parametric feature tree.
///
/// Serialization format is designed for clean git diffs:
/// - `id`: human-readable string like "extrude-1"
/// - `suppressed`: persisted flag (replaces FeatureState for serialization)
/// - `state`: runtime-only, recomputed on load
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Feature {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: FeatureKind,
    #[serde(default)]
    pub suppressed: bool,
    pub params: FeatureParams,
    #[serde(skip)]
    pub state: FeatureState,
}

impl Feature {
    pub fn new(id: String, name: String, kind: FeatureKind, params: FeatureParams) -> Self {
        Self {
            id,
            name,
            kind,
            params,
            suppressed: false,
            state: FeatureState::Pending,
        }
    }

    pub fn is_active(&self) -> bool {
        !self.suppressed
    }
}

impl Default for FeatureState {
    fn default() -> Self {
        FeatureState::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_defaults_to_pending() {
        let f = Feature::new(
            "extrude-1".into(),
            "Test".into(),
            FeatureKind::Extrude,
            FeatureParams::Placeholder,
        );
        assert_eq!(f.state, FeatureState::Pending);
        assert!(f.is_active());
        assert!(!f.suppressed);
    }

    #[test]
    fn suppressed_feature_is_not_active() {
        let mut f = Feature::new(
            "extrude-1".into(),
            "Test".into(),
            FeatureKind::Extrude,
            FeatureParams::Placeholder,
        );
        f.suppressed = true;
        assert!(!f.is_active());
    }

    #[test]
    fn state_not_serialized() {
        let mut f = Feature::new(
            "extrude-1".into(),
            "Test".into(),
            FeatureKind::Extrude,
            FeatureParams::Placeholder,
        );
        f.state = FeatureState::Evaluated;
        let json = serde_json::to_string(&f).unwrap();
        assert!(!json.contains("Evaluated"));
        assert!(!json.contains("state"));

        // Deserialize back — state should be Pending (default)
        let f2: Feature = serde_json::from_str(&json).unwrap();
        assert_eq!(f2.state, FeatureState::Pending);
    }
}
