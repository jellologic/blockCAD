use crate::id::EntityId;

use super::entity::SketchEntityId;

pub type ConstraintId = EntityId<Constraint>;

/// The kind of geometric or dimensional constraint.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ConstraintKind {
    // Geometric constraints
    Coincident,
    Collinear,
    Parallel,
    Perpendicular,
    Tangent,
    Symmetric { axis: SketchEntityId },
    Midpoint,

    // Dimensional constraints
    Distance { value: f64 },
    Angle { value: f64, supplementary: bool },
    Radius { value: f64 },
    Diameter { value: f64 },

    // Fix constraints
    Fixed,
    Horizontal,
    Vertical,

    // Equality
    Equal,

    // Advanced geometric constraints
    /// Two arcs/circles share the same center point.
    Concentric,
    /// Two arcs/circles share the same center and radius.
    Coradial,
    /// A point lies on a line or arc/circle curve.
    PointOnCurve,
}

/// A constraint applied to one or more sketch entities.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Constraint {
    pub kind: ConstraintKind,
    /// The sketch entities this constraint applies to (1–3 entities)
    pub entities: Vec<SketchEntityId>,
    /// Whether this is a driven (reference) dimension
    pub driven: bool,
}

impl Constraint {
    pub fn new(kind: ConstraintKind, entities: Vec<SketchEntityId>) -> Self {
        Self {
            kind,
            entities,
            driven: false,
        }
    }

    pub fn driven(mut self) -> Self {
        self.driven = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_creation() {
        let id1 = SketchEntityId::new(0, 0);
        let id2 = SketchEntityId::new(1, 0);
        let c = Constraint::new(ConstraintKind::Coincident, vec![id1, id2]);
        assert!(!c.driven);
        assert_eq!(c.entities.len(), 2);
    }

    #[test]
    fn driven_constraint() {
        let id1 = SketchEntityId::new(0, 0);
        let id2 = SketchEntityId::new(1, 0);
        let c = Constraint::new(ConstraintKind::Distance { value: 10.0 }, vec![id1, id2]).driven();
        assert!(c.driven);
    }
}
