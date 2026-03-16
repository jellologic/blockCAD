use crate::geometry::surface::plane::Plane;
use crate::topology::EntityStore;

use super::constraint::{Constraint, ConstraintId};
use super::entity::{SketchEntity, SketchEntityId};

/// A 2D parametric sketch on a plane.
/// Sketches contain entities (points, lines, arcs, etc.) and constraints
/// that define geometric relationships between them.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Sketch {
    pub plane: Plane,
    pub entities: EntityStore<SketchEntity>,
    pub constraints: EntityStore<Constraint>,
    /// Entity indices marked as construction geometry (excluded from profile extraction).
    #[serde(default)]
    pub construction_entities: std::collections::HashSet<usize>,
}

impl Sketch {
    pub fn new(plane: Plane) -> Self {
        Self {
            plane,
            entities: EntityStore::new(),
            constraints: EntityStore::new(),
            construction_entities: std::collections::HashSet::new(),
        }
    }

    /// Mark an entity as construction geometry (won't form profile edges).
    pub fn set_construction(&mut self, entity_index: usize, is_construction: bool) {
        if is_construction {
            self.construction_entities.insert(entity_index);
        } else {
            self.construction_entities.remove(&entity_index);
        }
    }

    /// Check if an entity is construction geometry.
    pub fn is_construction(&self, entity_index: usize) -> bool {
        self.construction_entities.contains(&entity_index)
    }

    pub fn add_entity(&mut self, entity: SketchEntity) -> SketchEntityId {
        self.entities.insert(entity)
    }

    pub fn add_constraint(&mut self, constraint: Constraint) -> ConstraintId {
        self.constraints.insert(constraint)
    }

    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pt2;

    #[test]
    fn sketch_add_entities() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let _line = sketch.add_entity(SketchEntity::Line {
            start: p1,
            end: p2,
        });
        assert_eq!(sketch.entity_count(), 3);
    }

    #[test]
    fn sketch_add_constraint() {
        use super::super::constraint::ConstraintKind;

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 1.0 },
            vec![p1, p2],
        ));
        assert_eq!(sketch.constraint_count(), 1);
    }
}
