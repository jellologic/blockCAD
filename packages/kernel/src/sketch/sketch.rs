use crate::geometry::surface::plane::Plane;
use crate::topology::EntityStore;

use super::block::{SketchBlock, SketchBlockInstance};
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
    /// Block definitions (reusable entity groups).
    #[serde(default)]
    pub block_definitions: Vec<SketchBlock>,
    /// Block instances placed in the sketch.
    #[serde(default)]
    pub block_instances: Vec<SketchBlockInstance>,
}

impl Sketch {
    pub fn new(plane: Plane) -> Self {
        Self {
            plane,
            entities: EntityStore::new(),
            constraints: EntityStore::new(),
            construction_entities: std::collections::HashSet::new(),
            block_definitions: Vec::new(),
            block_instances: Vec::new(),
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

    /// Add a block definition to the sketch.
    pub fn add_block(&mut self, block: SketchBlock) {
        self.block_definitions.push(block);
    }

    /// Add a block instance to the sketch.
    pub fn add_block_instance(&mut self, instance: SketchBlockInstance) {
        self.block_instances.push(instance);
    }

    /// Find a block definition by ID.
    pub fn find_block(&self, block_id: &str) -> Option<&SketchBlock> {
        self.block_definitions.iter().find(|b| b.id == block_id)
    }

    /// Remove a block definition and all its instances.
    pub fn remove_block(&mut self, block_id: &str) {
        self.block_definitions.retain(|b| b.id != block_id);
        self.block_instances.retain(|i| i.block_id != block_id);
    }

    /// Explode a block instance back into individual entities.
    /// Returns the entity IDs that were part of the block.
    pub fn explode_block_instance(&mut self, instance_id: &str) -> Vec<SketchEntityId> {
        let instance = self.block_instances.iter().find(|i| i.id == instance_id).cloned();
        if let Some(inst) = instance {
            if let Some(block) = self.find_block(&inst.block_id).cloned() {
                self.block_instances.retain(|i| i.id != instance_id);
                return block.entity_indices;
            }
        }
        Vec::new()
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

    #[test]
    fn sketch_add_block() {
        use super::super::block::{SketchBlock, SketchBlockInstance};

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let block = SketchBlock {
            id: "block-1".into(),
            name: "MyBlock".into(),
            insertion_point: Pt2::new(0.0, 0.0),
            entity_indices: vec![p1, p2, line],
        };
        sketch.add_block(block);

        assert_eq!(sketch.block_definitions.len(), 1);
        assert_eq!(sketch.block_definitions[0].id, "block-1");
        assert_eq!(sketch.block_definitions[0].name, "MyBlock");
        assert_eq!(sketch.block_definitions[0].entity_indices.len(), 3);
    }

    #[test]
    fn sketch_add_block_instance() {
        use super::super::block::{SketchBlock, SketchBlockInstance};

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });

        let block = SketchBlock {
            id: "block-1".into(),
            name: "MyBlock".into(),
            insertion_point: Pt2::new(0.0, 0.0),
            entity_indices: vec![p1],
        };
        sketch.add_block(block);

        let instance = SketchBlockInstance {
            id: "inst-1".into(),
            block_id: "block-1".into(),
            position: Pt2::new(5.0, 5.0),
            scale: 1.0,
            rotation: 0.0,
        };
        sketch.add_block_instance(instance);

        assert_eq!(sketch.block_instances.len(), 1);
        assert_eq!(sketch.block_instances[0].id, "inst-1");
        assert_eq!(sketch.block_instances[0].block_id, "block-1");
    }

    #[test]
    fn sketch_find_block() {
        use super::super::block::{SketchBlock, SketchBlockInstance};

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });

        let block = SketchBlock {
            id: "block-a".into(),
            name: "BlockA".into(),
            insertion_point: Pt2::new(0.0, 0.0),
            entity_indices: vec![p1],
        };
        sketch.add_block(block);

        // Should find an existing block
        let found = sketch.find_block("block-a");
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.id, "block-a");
        assert_eq!(found.name, "BlockA");

        // Should return None for non-existent block
        assert!(sketch.find_block("block-nonexistent").is_none());
    }

    #[test]
    fn sketch_remove_block() {
        use super::super::block::{SketchBlock, SketchBlockInstance};

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });

        let block = SketchBlock {
            id: "block-r".into(),
            name: "Removable".into(),
            insertion_point: Pt2::new(0.0, 0.0),
            entity_indices: vec![p1],
        };
        sketch.add_block(block);

        let inst1 = SketchBlockInstance {
            id: "inst-r1".into(),
            block_id: "block-r".into(),
            position: Pt2::new(1.0, 1.0),
            scale: 1.0,
            rotation: 0.0,
        };
        let inst2 = SketchBlockInstance {
            id: "inst-r2".into(),
            block_id: "block-r".into(),
            position: Pt2::new(2.0, 2.0),
            scale: 1.0,
            rotation: 0.0,
        };
        sketch.add_block_instance(inst1);
        sketch.add_block_instance(inst2);

        assert_eq!(sketch.block_definitions.len(), 1);
        assert_eq!(sketch.block_instances.len(), 2);

        sketch.remove_block("block-r");

        assert_eq!(sketch.block_definitions.len(), 0);
        assert_eq!(sketch.block_instances.len(), 0);
        assert!(sketch.find_block("block-r").is_none());
    }

    #[test]
    fn sketch_explode_block_instance() {
        use super::super::block::{SketchBlock, SketchBlockInstance};

        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let block = SketchBlock {
            id: "block-e".into(),
            name: "Explodable".into(),
            insertion_point: Pt2::new(0.0, 0.0),
            entity_indices: vec![p1, p2, line],
        };
        sketch.add_block(block);

        let instance = SketchBlockInstance {
            id: "inst-e1".into(),
            block_id: "block-e".into(),
            position: Pt2::new(5.0, 5.0),
            scale: 2.0,
            rotation: 0.0,
        };
        sketch.add_block_instance(instance);

        assert_eq!(sketch.block_instances.len(), 1);

        let entity_ids = sketch.explode_block_instance("inst-e1");

        // Instance should be removed
        assert_eq!(sketch.block_instances.len(), 0);
        // Block definition should still exist
        assert!(sketch.find_block("block-e").is_some());
        // Returned entity IDs should match the block's entity indices
        assert_eq!(entity_ids.len(), 3);
        assert_eq!(entity_ids[0], p1);
        assert_eq!(entity_ids[1], p2);
        assert_eq!(entity_ids[2], line);
    }
}
