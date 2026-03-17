use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt2;
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Parameters for creating a circular (rotational) pattern of sketch entities.
pub struct CircularSketchPatternParams {
    /// IDs of the entities to pattern.
    pub entity_ids: Vec<SketchEntityId>,
    /// Center point of rotation.
    pub center: Pt2,
    /// Total angle span in radians (use `2 * PI` for a full circle).
    pub total_angle: f64,
    /// Total number of copies **including** the original.
    pub count: u32,
    /// When `true`, copies are equally spaced over `total_angle`.
    /// When `false`, `total_angle` is the angle between each successive copy.
    pub equal_spacing: bool,
}

/// Result of a circular sketch pattern operation.
pub struct CircularSketchPatternResult {
    /// One `Vec<SketchEntityId>` per generated copy (excludes the originals).
    /// `pattern_entities[i]` corresponds to copy `i+1`.
    pub pattern_entities: Vec<Vec<SketchEntityId>>,
}

/// Rotate a 2D point around a center by the given angle (radians, CCW positive).
pub fn rotate_point(point: Pt2, center: Pt2, angle: f64) -> Pt2 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();
    let dx = point.x - center.x;
    let dy = point.y - center.y;
    Pt2::new(center.x + dx * cos_a - dy * sin_a, center.y + dx * sin_a + dy * cos_a)
}

/// Rotate a sketch entity around `center` by `angle` radians.
///
/// Point entities are rotated directly. For entities that reference other entities
/// by ID (Line, Arc, Circle, Spline, Ellipse), the caller must supply `id_map`
/// that maps original entity IDs to their already-rotated copies.
pub fn rotate_entity(
    entity: &SketchEntity,
    center: Pt2,
    angle: f64,
    id_map: &HashMap<SketchEntityId, SketchEntityId>,
) -> SketchEntity {
    match entity {
        SketchEntity::Point { position } => SketchEntity::Point {
            position: rotate_point(*position, center, angle),
        },
        SketchEntity::Line { start, end } => SketchEntity::Line {
            start: id_map[start],
            end: id_map[end],
        },
        SketchEntity::Arc {
            center: arc_center,
            start,
            end,
        } => SketchEntity::Arc {
            center: id_map[arc_center],
            start: id_map[start],
            end: id_map[end],
        },
        SketchEntity::Circle {
            center: circle_center,
            radius,
        } => SketchEntity::Circle {
            center: id_map[circle_center],
            radius: *radius,
        },
        SketchEntity::Spline {
            control_points,
            degree,
        } => SketchEntity::Spline {
            control_points: control_points.iter().map(|id| id_map[id]).collect(),
            degree: *degree,
        },
        SketchEntity::Ellipse {
            center: ellipse_center,
            radius_x,
            radius_y,
            rotation,
        } => SketchEntity::Ellipse {
            center: id_map[ellipse_center],
            radius_x: *radius_x,
            radius_y: *radius_y,
            rotation: rotation + angle,
        },
    }
}

/// Create rotated copies of the given sketch entities arranged in a circular pattern.
///
/// The original entities are kept as-is; only the new copies are added to the sketch.
/// Returns a [`CircularSketchPatternResult`] containing the IDs of all newly created entities.
pub fn circular_sketch_pattern(
    sketch: &mut Sketch,
    params: &CircularSketchPatternParams,
) -> KernelResult<CircularSketchPatternResult> {
    if params.count < 2 {
        return Err(KernelError::InvalidParameter {
            param: "count".into(),
            value: params.count.to_string(),
        });
    }

    if params.entity_ids.is_empty() {
        return Err(KernelError::InvalidParameter {
            param: "entity_ids".into(),
            value: "empty".into(),
        });
    }

    // Compute the angle step between successive copies.
    let angle_step = if params.equal_spacing {
        params.total_angle / params.count as f64
    } else {
        params.total_angle
    };

    // Classify entities: points first so their rotated copies exist before
    // non-point entities try to reference them via id_map.
    let mut point_ids = Vec::new();
    let mut non_point_ids = Vec::new();
    for &eid in &params.entity_ids {
        let entity = sketch.entities.get(eid)?;
        match entity {
            SketchEntity::Point { .. } => point_ids.push(eid),
            _ => non_point_ids.push(eid),
        }
    }

    // Snapshot the original entities so we can create rotated copies without
    // borrow conflicts.
    let mut original_entities: HashMap<SketchEntityId, SketchEntity> = HashMap::new();
    for &eid in point_ids.iter().chain(non_point_ids.iter()) {
        let entity = sketch.entities.get(eid)?.clone();
        original_entities.insert(eid, entity);
    }

    let num_copies = params.count - 1; // exclude the original
    let mut pattern_entities: Vec<Vec<SketchEntityId>> = Vec::with_capacity(num_copies as usize);

    for copy_idx in 1..=num_copies {
        let angle = angle_step * copy_idx as f64;
        let mut id_map: HashMap<SketchEntityId, SketchEntityId> = HashMap::new();
        let mut copy_ids: Vec<SketchEntityId> = Vec::new();

        // First pass: rotate point entities.
        for &eid in &point_ids {
            let original = &original_entities[&eid];
            let rotated = rotate_entity(original, params.center, angle, &id_map);
            let new_id = sketch.add_entity(rotated);
            id_map.insert(eid, new_id);
            copy_ids.push(new_id);
        }

        // Second pass: rotate non-point entities (which may reference point IDs).
        for &eid in &non_point_ids {
            let original = &original_entities[&eid];
            let rotated = rotate_entity(original, params.center, angle, &id_map);
            let new_id = sketch.add_entity(rotated);
            id_map.insert(eid, new_id);
            copy_ids.push(new_id);
        }

        pattern_entities.push(copy_ids);
    }

    Ok(CircularSketchPatternResult { pattern_entities })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use std::f64::consts::PI;

    const TOLERANCE: f64 = 1e-9;

    fn assert_pt_near(a: Pt2, b: Pt2) {
        assert!(
            (a.x - b.x).abs() < TOLERANCE && (a.y - b.y).abs() < TOLERANCE,
            "Points not near: ({}, {}) vs ({}, {})",
            a.x,
            a.y,
            b.x,
            b.y,
        );
    }

    fn get_point_position(sketch: &Sketch, id: SketchEntityId) -> Pt2 {
        match sketch.entities.get(id).unwrap() {
            SketchEntity::Point { position } => *position,
            _ => panic!("Expected a Point entity"),
        }
    }

    #[test]
    fn pattern_point_four_copies_at_90_degrees() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![pt],
                center: Pt2::new(0.0, 0.0),
                total_angle: 2.0 * PI,
                count: 4,
                equal_spacing: true,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 3);

        let p1 = get_point_position(&sketch, result.pattern_entities[0][0]);
        let p2 = get_point_position(&sketch, result.pattern_entities[1][0]);
        let p3 = get_point_position(&sketch, result.pattern_entities[2][0]);

        assert_pt_near(p1, Pt2::new(0.0, 1.0));
        assert_pt_near(p2, Pt2::new(-1.0, 0.0));
        assert_pt_near(p3, Pt2::new(0.0, -1.0));
    }

    #[test]
    fn pattern_full_circle() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 0.0),
        });

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![pt],
                center: Pt2::new(0.0, 0.0),
                total_angle: 2.0 * PI,
                count: 6,
                equal_spacing: true,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 5);

        // All copies should be at radius 3 from center.
        for copy in &result.pattern_entities {
            let pos = get_point_position(&sketch, copy[0]);
            let dist = ((pos.x).powi(2) + (pos.y).powi(2)).sqrt();
            assert!((dist - 3.0).abs() < TOLERANCE);
        }

        // Last copy should be near (but not at) the original (since it is
        // at 5/6 * 2*PI, not 2*PI itself).
        let last = get_point_position(&sketch, result.pattern_entities[4][0]);
        let expected_angle = 5.0 / 6.0 * 2.0 * PI;
        assert_pt_near(
            last,
            Pt2::new(3.0 * expected_angle.cos(), 3.0 * expected_angle.sin()),
        );
    }

    #[test]
    fn pattern_partial_angle_180_three_copies() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(2.0, 0.0),
        });

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![pt],
                center: Pt2::new(0.0, 0.0),
                total_angle: PI,
                count: 3,
                equal_spacing: true,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 2);

        // 60 degrees
        let p1 = get_point_position(&sketch, result.pattern_entities[0][0]);
        assert_pt_near(
            p1,
            Pt2::new(2.0 * (PI / 3.0).cos(), 2.0 * (PI / 3.0).sin()),
        );

        // 120 degrees
        let p2 = get_point_position(&sketch, result.pattern_entities[1][0]);
        assert_pt_near(
            p2,
            Pt2::new(2.0 * (2.0 * PI / 3.0).cos(), 2.0 * (2.0 * PI / 3.0).sin()),
        );
    }

    #[test]
    fn pattern_line_segment() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(2.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![p1, p2, line],
                center: Pt2::new(0.0, 0.0),
                total_angle: 2.0 * PI,
                count: 4,
                equal_spacing: true,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 3);

        // Check the first copy (90 degrees).
        let copy = &result.pattern_entities[0];
        assert_eq!(copy.len(), 3); // 2 points + 1 line

        let cp1 = get_point_position(&sketch, copy[0]);
        let cp2 = get_point_position(&sketch, copy[1]);
        assert_pt_near(cp1, Pt2::new(0.0, 1.0));
        assert_pt_near(cp2, Pt2::new(0.0, 2.0));

        // Verify the line references the rotated points.
        match sketch.entities.get(copy[2]).unwrap() {
            SketchEntity::Line { start, end } => {
                assert_eq!(*start, copy[0]);
                assert_eq!(*end, copy[1]);
            }
            _ => panic!("Expected a Line entity"),
        }
    }

    #[test]
    fn pattern_arc() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let center_pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let start_pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let end_pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 1.0),
        });
        let arc = sketch.add_entity(SketchEntity::Arc {
            center: center_pt,
            start: start_pt,
            end: end_pt,
        });

        // 2 copies (including original) over PI with equal spacing => step = PI/2
        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![center_pt, start_pt, end_pt, arc],
                center: Pt2::new(0.0, 0.0),
                total_angle: PI,
                count: 2,
                equal_spacing: true,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 1);
        let copy = &result.pattern_entities[0];
        assert_eq!(copy.len(), 4); // 3 points + 1 arc

        // Center should rotate to (0, 0) (it's at the rotation center).
        let rotated_center = get_point_position(&sketch, copy[0]);
        assert_pt_near(rotated_center, Pt2::new(0.0, 0.0));

        // Start (1,0) rotated 90 -> (0, 1)
        let rotated_start = get_point_position(&sketch, copy[1]);
        assert_pt_near(rotated_start, Pt2::new(0.0, 1.0));

        // End (0,1) rotated 90 -> (-1, 0)
        let rotated_end = get_point_position(&sketch, copy[2]);
        assert_pt_near(rotated_end, Pt2::new(-1.0, 0.0));

        // Verify arc references.
        match sketch.entities.get(copy[3]).unwrap() {
            SketchEntity::Arc { center, start, end } => {
                assert_eq!(*center, copy[0]);
                assert_eq!(*start, copy[1]);
                assert_eq!(*end, copy[2]);
            }
            _ => panic!("Expected an Arc entity"),
        }
    }

    #[test]
    fn pattern_non_equal_spacing() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });

        // With equal_spacing=false, total_angle is the step between each copy.
        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![pt],
                center: Pt2::new(0.0, 0.0),
                total_angle: PI / 4.0, // 45 degrees between each
                count: 3,
                equal_spacing: false,
            },
        )
        .unwrap();

        assert_eq!(result.pattern_entities.len(), 2);

        // First copy at 45 degrees.
        let p1 = get_point_position(&sketch, result.pattern_entities[0][0]);
        assert_pt_near(
            p1,
            Pt2::new((PI / 4.0).cos(), (PI / 4.0).sin()),
        );

        // Second copy at 90 degrees.
        let p2 = get_point_position(&sketch, result.pattern_entities[1][0]);
        assert_pt_near(
            p2,
            Pt2::new((PI / 2.0).cos(), (PI / 2.0).sin()),
        );
    }

    #[test]
    fn pattern_count_less_than_two_errors() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![pt],
                center: Pt2::new(0.0, 0.0),
                total_angle: PI,
                count: 1,
                equal_spacing: true,
            },
        );

        assert!(result.is_err());
    }

    #[test]
    fn pattern_empty_entity_ids_errors() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let result = circular_sketch_pattern(
            &mut sketch,
            &CircularSketchPatternParams {
                entity_ids: vec![],
                center: Pt2::new(0.0, 0.0),
                total_angle: PI,
                count: 3,
                equal_spacing: true,
            },
        );

        assert!(result.is_err());
    }
}
