use super::entity::SketchEntityId;

/// Dimension types for sketch annotation and constraint driving.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Dimension {
    /// Linear distance between two points or a point and a line
    Linear {
        entity_a: SketchEntityId,
        entity_b: SketchEntityId,
        value: f64,
    },
    /// Angular dimension between two lines
    Angular {
        line_a: SketchEntityId,
        line_b: SketchEntityId,
        value: f64,
    },
    /// Radius of a circle or arc
    Radial {
        entity: SketchEntityId,
        value: f64,
    },
    /// Diameter of a circle or arc
    Diametral {
        entity: SketchEntityId,
        value: f64,
    },
}
