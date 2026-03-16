use crate::geometry::Pt2;
use crate::id::EntityId;

pub type SketchEntityId = EntityId<SketchEntity>;

/// Sketch entities live in a 2D plane coordinate system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SketchEntity {
    Point {
        position: Pt2,
    },
    Line {
        start: SketchEntityId,
        end: SketchEntityId,
    },
    Arc {
        center: SketchEntityId,
        start: SketchEntityId,
        end: SketchEntityId,
    },
    Circle {
        center: SketchEntityId,
        radius: f64,
    },
    Spline {
        control_points: Vec<SketchEntityId>,
        degree: usize,
    },
    /// Ellipse defined by center point, two radii, and rotation angle.
    Ellipse {
        center: SketchEntityId,
        radius_x: f64,
        radius_y: f64,
        /// Rotation of the major axis from the sketch U-axis (radians).
        rotation: f64,
    },
}
