use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::geometry::Vec2;
use crate::id::EntityId;
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Parameters for creating a linear pattern of sketch entities.
#[derive(Debug, Clone)]
pub struct LinearSketchPatternParams {
    /// IDs of entities to pattern.
    pub entity_ids: Vec<SketchEntityId>,
    /// Primary direction vector (will be normalized internally).
    pub direction: Vec2,
    /// Spacing between instances along the primary direction.
    pub spacing: f64,
    /// Number of instances along the primary direction (including the original).
    pub count: u32,
    /// Optional second direction for a 2D grid pattern.
    pub direction2: Option<Vec2>,
    /// Spacing along the second direction.
    pub spacing2: Option<f64>,
    /// Number of instances along the second direction (including the original).
    pub count2: Option<u32>,
}

/// Result of creating a linear sketch pattern.
#[derive(Debug, Clone)]
pub struct LinearSketchPatternResult {
    /// Newly created entity IDs, grouped by pattern instance.
    /// Does not include the original entities.
    /// For a 1D pattern with count=3, there are 2 instance groups.
    /// For a 2D pattern with count=3, count2=2, there are (3*2 - 1) instance groups.
    pub pattern_entities: Vec<Vec<SketchEntityId>>,
}

/// Create a linear pattern of sketch entities.
///
/// Copies the specified entities `count - 1` times along `direction` with the given
/// `spacing`. If second-direction parameters are provided, creates a 2D grid pattern.
///
/// The original entities (at offset 0) are not duplicated. Each new instance is a
/// translated copy that preserves internal connectivity (e.g., a Line's start/end
/// point references are remapped to the copied points).
pub fn linear_sketch_pattern(
    sketch: &mut Sketch,
    params: &LinearSketchPatternParams,
) -> KernelResult<LinearSketchPatternResult> {
    // Validate parameters
    if params.count == 0 {
        return Err(KernelError::InvalidParameter {
            param: "count".into(),
            value: "0".into(),
        });
    }
    if params.spacing < 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "spacing".into(),
            value: params.spacing.to_string(),
        });
    }
    let dir_norm = params.direction.norm();
    if dir_norm < 1e-12 {
        return Err(KernelError::InvalidParameter {
            param: "direction".into(),
            value: format!("{:?} (zero-length)", params.direction),
        });
    }
    let dir1 = params.direction / dir_norm;

    let count2 = params.count2.unwrap_or(1);
    let dir2 = if count2 > 1 {
        let d2 = params.direction2.ok_or_else(|| KernelError::InvalidParameter {
            param: "direction2".into(),
            value: "None (required when count2 > 1)".into(),
        })?;
        let d2_norm = d2.norm();
        if d2_norm < 1e-12 {
            return Err(KernelError::InvalidParameter {
                param: "direction2".into(),
                value: format!("{:?} (zero-length)", d2),
            });
        }
        Some(d2 / d2_norm)
    } else {
        None
    };
    let spacing2 = params.spacing2.unwrap_or(0.0);

    // Validate that all referenced entities exist by reading them.
    // Collect the source entities so we can clone them for each instance.
    let source_entities: Vec<(SketchEntityId, SketchEntity)> = params
        .entity_ids
        .iter()
        .map(|&id| {
            let entity = sketch.entities.get(id)?;
            Ok((id, entity.clone()))
        })
        .collect::<KernelResult<Vec<_>>>()?;

    let mut all_instances = Vec::new();

    for j in 0..count2 {
        for i in 0..params.count {
            // Skip the original position (i=0, j=0).
            if i == 0 && j == 0 {
                continue;
            }

            let offset = dir1 * (params.spacing * i as f64)
                + dir2.unwrap_or(Vec2::zeros()) * (spacing2 * j as f64);

            let instance_ids =
                create_translated_instance(sketch, &source_entities, offset)?;
            all_instances.push(instance_ids);
        }
    }

    Ok(LinearSketchPatternResult {
        pattern_entities: all_instances,
    })
}

/// Create one translated copy of the given source entities.
///
/// Returns the new entity IDs in the same order as `source_entities`.
/// Internal references (e.g., Line start/end pointing to source points)
/// are remapped to the newly created copies.
fn create_translated_instance(
    sketch: &mut Sketch,
    source_entities: &[(SketchEntityId, SketchEntity)],
    offset: Vec2,
) -> KernelResult<Vec<SketchEntityId>> {
    // First pass: create placeholder points so we can build an ID mapping.
    // We need the mapping before we can translate entities that reference other entities.
    let mut id_map: HashMap<SketchEntityId, SketchEntityId> = HashMap::new();
    let mut new_ids: Vec<SketchEntityId> = Vec::with_capacity(source_entities.len());

    // First pass: insert all entities as placeholders (points get their final value,
    // others get a dummy that will be replaced).
    for (old_id, entity) in source_entities {
        let new_entity = translate_entity(entity, offset, &id_map);
        let new_id = sketch.add_entity(new_entity);
        id_map.insert(*old_id, new_id);
        new_ids.push(new_id);
    }

    // Second pass: fix up entities that reference other entities, now that all IDs
    // are in the map. Only needed for entities whose references weren't yet available
    // during the first pass.
    for (idx, (_old_id, entity)) in source_entities.iter().enumerate() {
        if needs_remap(entity) {
            let corrected = translate_entity(entity, offset, &id_map);
            let target_id = new_ids[idx];
            *sketch.entities.get_mut(target_id)? = corrected;
        }
    }

    Ok(new_ids)
}

/// Translate a sketch entity by the given offset, remapping any internal entity
/// references using the provided ID map.
fn translate_entity(
    entity: &SketchEntity,
    offset: Vec2,
    id_map: &HashMap<SketchEntityId, SketchEntityId>,
) -> SketchEntity {
    match entity {
        SketchEntity::Point { position } => SketchEntity::Point {
            position: *position + offset,
        },
        SketchEntity::Line { start, end } => SketchEntity::Line {
            start: remap(*start, id_map),
            end: remap(*end, id_map),
        },
        SketchEntity::Arc { center, start, end } => SketchEntity::Arc {
            center: remap(*center, id_map),
            start: remap(*start, id_map),
            end: remap(*end, id_map),
        },
        SketchEntity::Circle { center, radius } => SketchEntity::Circle {
            center: remap(*center, id_map),
            radius: *radius,
        },
        SketchEntity::Spline {
            control_points,
            degree,
        } => SketchEntity::Spline {
            control_points: control_points.iter().map(|id| remap(*id, id_map)).collect(),
            degree: *degree,
        },
        SketchEntity::Ellipse {
            center,
            radius_x,
            radius_y,
            rotation,
        } => SketchEntity::Ellipse {
            center: remap(*center, id_map),
            radius_x: *radius_x,
            radius_y: *radius_y,
            rotation: *rotation,
        },
    }
}

/// Remap an entity ID through the map, falling back to the original if not found.
fn remap(id: SketchEntityId, map: &HashMap<SketchEntityId, SketchEntityId>) -> SketchEntityId {
    map.get(&id).copied().unwrap_or(id)
}

/// Returns true if the entity contains references to other entities that may need remapping.
fn needs_remap(entity: &SketchEntity) -> bool {
    !matches!(entity, SketchEntity::Point { .. })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Pt2, Vec2};
    use crate::sketch::sketch::Sketch;
    use crate::sketch::entity::SketchEntity;
    use crate::geometry::surface::plane::Plane;

    /// Helper: build a sketch with a single line (two points + line entity).
    fn sketch_with_line() -> (Sketch, SketchEntityId, SketchEntityId, SketchEntityId) {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        (sketch, p1, p2, line)
    }

    /// Helper: build a sketch with a rectangle (4 points + 4 lines).
    fn sketch_with_rectangle() -> (Sketch, Vec<SketchEntityId>) {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 1.0) });
        let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 1.0) });
        let l0 = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let l1 = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let l2 = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let l3 = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
        (sketch, vec![p0, p1, p2, p3, l0, l1, l2, l3])
    }

    fn pt_position(sketch: &Sketch, id: SketchEntityId) -> Pt2 {
        match sketch.entities.get(id).unwrap() {
            SketchEntity::Point { position } => *position,
            _ => panic!("expected Point entity"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 1: 1D linear pattern of a single line (3 copies total)
    // -----------------------------------------------------------------------
    #[test]
    fn test_1d_pattern_single_line() {
        let (mut sketch, p1, p2, line) = sketch_with_line();

        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 3,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        )
        .unwrap();

        // 3 instances total, minus the original = 2 new instance groups.
        assert_eq!(result.pattern_entities.len(), 2);

        // Each instance should have 3 entities (2 points + 1 line).
        for instance in &result.pattern_entities {
            assert_eq!(instance.len(), 3);
        }

        // Check positions of first copy (offset = 5.0 in X).
        let inst1 = &result.pattern_entities[0];
        let pos_start = pt_position(&sketch, inst1[0]);
        let pos_end = pt_position(&sketch, inst1[1]);
        assert!((pos_start.x - 5.0).abs() < 1e-9);
        assert!((pos_start.y - 0.0).abs() < 1e-9);
        assert!((pos_end.x - 6.0).abs() < 1e-9);
        assert!((pos_end.y - 0.0).abs() < 1e-9);

        // Check positions of second copy (offset = 10.0 in X).
        let inst2 = &result.pattern_entities[1];
        let pos_start2 = pt_position(&sketch, inst2[0]);
        let pos_end2 = pt_position(&sketch, inst2[1]);
        assert!((pos_start2.x - 10.0).abs() < 1e-9);
        assert!((pos_end2.x - 11.0).abs() < 1e-9);

        // Verify connectivity: the line in each instance references its own points.
        match sketch.entities.get(inst1[2]).unwrap() {
            SketchEntity::Line { start, end } => {
                assert_eq!(*start, inst1[0]);
                assert_eq!(*end, inst1[1]);
            }
            _ => panic!("expected Line"),
        }
    }

    // -----------------------------------------------------------------------
    // Test 2: 1D pattern of connected entities (rectangle)
    // -----------------------------------------------------------------------
    #[test]
    fn test_1d_pattern_rectangle() {
        let (mut sketch, ids) = sketch_with_rectangle();

        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: ids.clone(),
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 2,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        )
        .unwrap();

        // 1 new instance (count=2 minus original).
        assert_eq!(result.pattern_entities.len(), 1);
        let inst = &result.pattern_entities[0];
        assert_eq!(inst.len(), 8); // 4 points + 4 lines

        // The first line of the copy should connect its own first two points.
        match sketch.entities.get(inst[4]).unwrap() {
            SketchEntity::Line { start, end } => {
                assert_eq!(*start, inst[0]);
                assert_eq!(*end, inst[1]);
            }
            _ => panic!("expected Line"),
        }

        // Verify the copied rectangle corner positions.
        let expected_positions = [
            Pt2::new(5.0, 0.0),
            Pt2::new(7.0, 0.0),
            Pt2::new(7.0, 1.0),
            Pt2::new(5.0, 1.0),
        ];
        for (i, expected) in expected_positions.iter().enumerate() {
            let pos = pt_position(&sketch, inst[i]);
            assert!(
                (pos.x - expected.x).abs() < 1e-9 && (pos.y - expected.y).abs() < 1e-9,
                "Point {i} mismatch: got ({}, {}), expected ({}, {})",
                pos.x,
                pos.y,
                expected.x,
                expected.y,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 3: 2D grid pattern
    // -----------------------------------------------------------------------
    #[test]
    fn test_2d_pattern_grid() {
        let (mut sketch, p1, p2, line) = sketch_with_line();

        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 3,
                direction2: Some(Vec2::new(0.0, 1.0)),
                spacing2: Some(3.0),
                count2: Some(2),
            },
        )
        .unwrap();

        // Total instances: 3 * 2 = 6, minus original = 5.
        assert_eq!(result.pattern_entities.len(), 5);

        // Collect all first-point positions across instances.
        let mut positions: Vec<(f64, f64)> = Vec::new();
        // Include original
        positions.push((0.0, 0.0));
        for inst in &result.pattern_entities {
            let pos = pt_position(&sketch, inst[0]);
            positions.push((pos.x, pos.y));
        }
        positions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap().then(a.1.partial_cmp(&b.1).unwrap()));

        let expected = vec![
            (0.0, 0.0),
            (0.0, 3.0),
            (5.0, 0.0),
            (5.0, 3.0),
            (10.0, 0.0),
            (10.0, 3.0),
        ];
        assert_eq!(positions.len(), expected.len());
        for (got, exp) in positions.iter().zip(expected.iter()) {
            assert!(
                (got.0 - exp.0).abs() < 1e-9 && (got.1 - exp.1).abs() < 1e-9,
                "Position mismatch: got {:?}, expected {:?}",
                got,
                exp,
            );
        }
    }

    // -----------------------------------------------------------------------
    // Test 4: count=1 produces no copies
    // -----------------------------------------------------------------------
    #[test]
    fn test_count_1_no_copies() {
        let (mut sketch, p1, p2, line) = sketch_with_line();
        let initial_count = sketch.entity_count();

        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 1,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        )
        .unwrap();

        assert!(result.pattern_entities.is_empty());
        assert_eq!(sketch.entity_count(), initial_count);
    }

    // -----------------------------------------------------------------------
    // Test 5: pattern with points and circles (non-line entities)
    // -----------------------------------------------------------------------
    #[test]
    fn test_pattern_with_circle() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let circle = sketch.add_entity(SketchEntity::Circle {
            center,
            radius: 2.5,
        });

        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![center, circle],
                direction: Vec2::new(0.0, 1.0),
                spacing: 10.0,
                count: 3,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 2);

        // Verify the circle copies reference their own centers.
        for (i, inst) in result.pattern_entities.iter().enumerate() {
            let expected_y = 10.0 * (i + 1) as f64;
            let pos = pt_position(&sketch, inst[0]);
            assert!((pos.y - expected_y).abs() < 1e-9);

            match sketch.entities.get(inst[1]).unwrap() {
                SketchEntity::Circle { center, radius } => {
                    assert_eq!(*center, inst[0]);
                    assert!((radius - 2.5).abs() < 1e-9);
                }
                _ => panic!("expected Circle"),
            }
        }
    }

    // -----------------------------------------------------------------------
    // Test 6: invalid parameters
    // -----------------------------------------------------------------------
    #[test]
    fn test_zero_count_is_error() {
        let (mut sketch, p1, p2, line) = sketch_with_line();
        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 0,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_direction_is_error() {
        let (mut sketch, p1, p2, line) = sketch_with_line();
        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                direction: Vec2::new(0.0, 0.0),
                spacing: 5.0,
                count: 2,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_nonexistent_entity_is_error() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let fake_id = EntityId::new(999, 0);
        let result = linear_sketch_pattern(
            &mut sketch,
            &LinearSketchPatternParams {
                entity_ids: vec![fake_id],
                direction: Vec2::new(1.0, 0.0),
                spacing: 5.0,
                count: 2,
                direction2: None,
                spacing2: None,
                count2: None,
            },
        );
        assert!(result.is_err());
    }
}
