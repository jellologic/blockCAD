//! Mirror entities sketch tool.
//!
//! Creates mirrored copies of sketch entities about a mirror line defined
//! either by an existing line entity or by two arbitrary points.

use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt2, Vec2};
use crate::sketch::constraint::{Constraint, ConstraintId, ConstraintKind};
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Specifies the mirror line.
#[derive(Debug, Clone)]
pub enum MirrorLine {
    /// A line entity in the sketch to mirror about.
    Entity(SketchEntityId),
    /// An arbitrary line defined by two points.
    TwoPoints(Pt2, Pt2),
}

/// Result of a mirror operation.
#[derive(Debug, Clone)]
pub struct MirrorResult {
    /// The newly created mirrored entity IDs (in the same order as the input).
    pub mirrored_entities: Vec<SketchEntityId>,
    /// Symmetric constraints added between original and mirrored entities (empty
    /// if `add_constraints` was false).
    pub constraints: Vec<ConstraintId>,
}

/// Reflect a 2D point across a line defined by `line_origin` and unit direction `line_dir`.
///
/// The reflection formula is: P' = 2 * proj_line(P - O) + O - (P - O)
/// which simplifies to: P' = 2(d . v)v + O - d   where d = P - O, v = line_dir (unit).
pub fn mirror_point(point: Pt2, line_origin: Pt2, line_dir: Vec2) -> Pt2 {
    let d = point - line_origin;
    let proj_scalar = d.dot(&line_dir);
    line_origin + 2.0 * proj_scalar * line_dir - d
}

/// Create a mirrored copy of a single sketch entity.
///
/// Point-reference entities (Line, Arc, Spline) have their point IDs remapped
/// through `id_map`, which maps original point entity IDs to their mirrored
/// counterparts. For arcs the start/end points are swapped to reverse the
/// sweep direction.
fn mirror_sketch_entity(
    entity: &SketchEntity,
    line_origin: Pt2,
    line_dir: Vec2,
    id_map: &HashMap<SketchEntityId, SketchEntityId>,
) -> SketchEntity {
    match entity {
        SketchEntity::Point { position } => SketchEntity::Point {
            position: mirror_point(*position, line_origin, line_dir),
        },
        SketchEntity::Line { start, end } => SketchEntity::Line {
            start: id_map[start],
            end: id_map[end],
        },
        SketchEntity::Arc { center, start, end } => SketchEntity::Arc {
            center: id_map[center],
            // Swap start/end to reverse arc direction after mirroring.
            start: id_map[end],
            end: id_map[start],
        },
        SketchEntity::Circle { center, radius } => SketchEntity::Circle {
            center: id_map[center],
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
            center,
            radius_x,
            radius_y,
            rotation,
        } => SketchEntity::Ellipse {
            center: id_map[center],
            radius_x: *radius_x,
            radius_y: *radius_y,
            // Mirroring negates the rotation angle about the mirror line.
            rotation: -rotation,
        },
    }
}

/// Returns true if the entity is a `Point`.
fn is_point(entity: &SketchEntity) -> bool {
    matches!(entity, SketchEntity::Point { .. })
}

/// Resolve the mirror line into (origin, unit_direction).
fn resolve_mirror_line(sketch: &Sketch, mirror_line: &MirrorLine) -> KernelResult<(Pt2, Vec2)> {
    match mirror_line {
        MirrorLine::TwoPoints(a, b) => {
            let dir = b - a;
            let len = dir.norm();
            if len < 1e-12 {
                return Err(KernelError::InvalidParameter {
                    param: "mirror_line".into(),
                    value: "two identical points".into(),
                });
            }
            Ok((*a, dir / len))
        }
        MirrorLine::Entity(line_id) => {
            let line_entity = sketch.entities.get(*line_id)?;
            match line_entity {
                SketchEntity::Line { start, end } => {
                    let start_entity = sketch.entities.get(*start)?;
                    let end_entity = sketch.entities.get(*end)?;
                    let (a, b) = match (start_entity, end_entity) {
                        (
                            SketchEntity::Point { position: pa },
                            SketchEntity::Point { position: pb },
                        ) => (*pa, *pb),
                        _ => {
                            return Err(KernelError::InvalidParameter {
                                param: "mirror_line".into(),
                                value: "line endpoints are not points".into(),
                            });
                        }
                    };
                    let dir = b - a;
                    let len = dir.norm();
                    if len < 1e-12 {
                        return Err(KernelError::InvalidParameter {
                            param: "mirror_line".into(),
                            value: "degenerate line entity (zero length)".into(),
                        });
                    }
                    Ok((a, dir / len))
                }
                _ => Err(KernelError::InvalidParameter {
                    param: "mirror_line".into(),
                    value: "entity is not a line".into(),
                }),
            }
        }
    }
}

/// Collect all point entity IDs that are referenced by the given entities.
/// These need to be mirrored first so that compound entities (lines, arcs, etc.)
/// can reference them.
fn collect_referenced_point_ids(
    sketch: &Sketch,
    entity_ids: &[SketchEntityId],
) -> KernelResult<Vec<SketchEntityId>> {
    let entity_set: std::collections::HashSet<SketchEntityId> =
        entity_ids.iter().copied().collect();
    let mut point_ids = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for &eid in entity_ids {
        let entity = sketch.entities.get(eid)?;
        let refs: Vec<SketchEntityId> = match entity {
            SketchEntity::Line { start, end } => vec![*start, *end],
            SketchEntity::Arc { center, start, end } => vec![*center, *start, *end],
            SketchEntity::Circle { center, .. } => vec![*center],
            SketchEntity::Spline {
                control_points, ..
            } => control_points.clone(),
            SketchEntity::Ellipse { center, .. } => vec![*center],
            SketchEntity::Point { .. } => continue,
        };
        for rid in refs {
            // Only auto-include referenced points that are NOT already in entity_ids
            // (those will be processed normally). This avoids duplicating points the
            // user explicitly included.
            if !entity_set.contains(&rid) && seen.insert(rid) {
                point_ids.push(rid);
            }
        }
    }
    Ok(point_ids)
}

/// Mirror a set of sketch entities about a mirror line.
///
/// # Arguments
/// * `sketch`          - The sketch to modify.
/// * `entity_ids`      - IDs of the entities to mirror.
/// * `mirror_line`     - The line to mirror about.
/// * `add_constraints` - When true, adds `Symmetric` constraints between each
///                        original point and its mirrored counterpart.
///
/// # Returns
/// A `MirrorResult` containing the new entity IDs and any added constraints.
pub fn mirror_entities(
    sketch: &mut Sketch,
    entity_ids: &[SketchEntityId],
    mirror_line: &MirrorLine,
    add_constraints: bool,
) -> KernelResult<MirrorResult> {
    if entity_ids.is_empty() {
        return Ok(MirrorResult {
            mirrored_entities: Vec::new(),
            constraints: Vec::new(),
        });
    }

    let (line_origin, line_dir) = resolve_mirror_line(sketch, mirror_line)?;

    // We need the mirror-line entity ID for Symmetric constraints.
    let mirror_line_entity_id = match mirror_line {
        MirrorLine::Entity(id) => Some(*id),
        MirrorLine::TwoPoints(..) => None,
    };

    // Collect implicitly referenced point IDs that need mirroring.
    let implicit_point_ids = collect_referenced_point_ids(sketch, entity_ids)?;

    // A map from original entity ID -> mirrored entity ID.
    let mut id_map: HashMap<SketchEntityId, SketchEntityId> = HashMap::new();

    // Phase 1: Mirror all implicit referenced points first.
    for &pid in &implicit_point_ids {
        let entity = sketch.entities.get(pid)?.clone();
        let mirrored = mirror_sketch_entity(&entity, line_origin, line_dir, &id_map);
        let new_id = sketch.add_entity(mirrored);
        id_map.insert(pid, new_id);
    }

    // Phase 2: Mirror all explicitly requested entities.
    // Process points first, then non-points, to ensure id_map is populated.
    let mut point_ids_in_input = Vec::new();
    let mut non_point_ids = Vec::new();
    for &eid in entity_ids {
        let entity = sketch.entities.get(eid)?;
        if is_point(entity) {
            point_ids_in_input.push(eid);
        } else {
            non_point_ids.push(eid);
        }
    }

    // Mirror explicit points.
    for &pid in &point_ids_in_input {
        let entity = sketch.entities.get(pid)?.clone();
        let mirrored = mirror_sketch_entity(&entity, line_origin, line_dir, &id_map);
        let new_id = sketch.add_entity(mirrored);
        id_map.insert(pid, new_id);
    }

    // Mirror non-point entities (they reference points via id_map).
    for &eid in &non_point_ids {
        let entity = sketch.entities.get(eid)?.clone();
        let mirrored = mirror_sketch_entity(&entity, line_origin, line_dir, &id_map);
        let new_id = sketch.add_entity(mirrored);
        id_map.insert(eid, new_id);
    }

    // Build the result list in the same order as entity_ids.
    let mirrored_entities: Vec<SketchEntityId> =
        entity_ids.iter().map(|eid| id_map[eid]).collect();

    // Phase 3: Optionally add Symmetric constraints for point pairs.
    let mut constraints = Vec::new();
    if add_constraints {
        if let Some(axis_id) = mirror_line_entity_id {
            // Add symmetric constraints for all mirrored points (implicit + explicit).
            let all_point_originals: Vec<SketchEntityId> = implicit_point_ids
                .iter()
                .chain(point_ids_in_input.iter())
                .copied()
                .collect();

            for &orig_id in &all_point_originals {
                let mirror_id = id_map[&orig_id];
                let constraint = Constraint::new(
                    ConstraintKind::Symmetric { axis: axis_id },
                    vec![orig_id, mirror_id],
                );
                let cid = sketch.add_constraint(constraint);
                constraints.push(cid);
            }
        }
        // If mirror_line is TwoPoints we cannot add Symmetric constraints
        // because there is no line entity to reference as the axis.
    }

    Ok(MirrorResult {
        mirrored_entities,
        constraints,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    fn assert_pt_eq(a: &Pt2, b: &Pt2) {
        assert!(
            approx_eq(a.x, b.x) && approx_eq(a.y, b.y),
            "Points differ: ({}, {}) vs ({}, {})",
            a.x,
            a.y,
            b.x,
            b.y
        );
    }

    fn get_point_position(sketch: &Sketch, id: SketchEntityId) -> Pt2 {
        match sketch.entities.get(id).unwrap() {
            SketchEntity::Point { position } => *position,
            _ => panic!("Expected Point entity"),
        }
    }

    // ---- mirror_point unit tests ----

    #[test]
    fn mirror_point_about_vertical_line() {
        // Mirror (3, 2) about the Y-axis (x=0).
        let result = mirror_point(
            Pt2::new(3.0, 2.0),
            Pt2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
        );
        assert_pt_eq(&result, &Pt2::new(-3.0, 2.0));
    }

    #[test]
    fn mirror_point_about_horizontal_line() {
        let result = mirror_point(
            Pt2::new(3.0, 2.0),
            Pt2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
        );
        assert_pt_eq(&result, &Pt2::new(3.0, -2.0));
    }

    #[test]
    fn mirror_point_about_diagonal_line() {
        // Mirror (1, 0) about y=x => (0, 1).
        let dir = Vec2::new(1.0, 1.0).normalize();
        let result = mirror_point(Pt2::new(1.0, 0.0), Pt2::new(0.0, 0.0), dir);
        assert_pt_eq(&result, &Pt2::new(0.0, 1.0));
    }

    #[test]
    fn mirror_point_on_line_is_invariant() {
        let result = mirror_point(
            Pt2::new(0.0, 5.0),
            Pt2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
        );
        assert_pt_eq(&result, &Pt2::new(0.0, 5.0));
    }

    // ---- Full mirror_entities tests ----

    fn make_sketch_with_mirror_line() -> (Sketch, SketchEntityId) {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        // Vertical mirror line at x = 0.
        let lp1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let lp2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 10.0),
        });
        let line_id = sketch.add_entity(SketchEntity::Line {
            start: lp1,
            end: lp2,
        });
        (sketch, line_id)
    }

    #[test]
    fn mirror_single_point_about_entity() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 3.0),
        });

        let result = mirror_entities(
            &mut sketch,
            &[pt],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 1);
        let mirrored_pos = get_point_position(&sketch, result.mirrored_entities[0]);
        assert_pt_eq(&mirrored_pos, &Pt2::new(-5.0, 3.0));
        assert!(result.constraints.is_empty());
    }

    #[test]
    fn mirror_line_about_angled_line() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Mirror line along y = x through origin.
        let mlp1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let mlp2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 1.0),
        });
        let ml = sketch.add_entity(SketchEntity::Line {
            start: mlp1,
            end: mlp2,
        });

        // A horizontal line from (2, 0) to (4, 0) mirrored about y=x => vertical line (0,2)-(0,4).
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(2.0, 0.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(4.0, 0.0),
        });
        let line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let result = mirror_entities(
            &mut sketch,
            &[p1, p2, line],
            &MirrorLine::Entity(ml),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 3);
        let mp1 = get_point_position(&sketch, result.mirrored_entities[0]);
        let mp2 = get_point_position(&sketch, result.mirrored_entities[1]);
        assert_pt_eq(&mp1, &Pt2::new(0.0, 2.0));
        assert_pt_eq(&mp2, &Pt2::new(0.0, 4.0));

        // Verify the mirrored line references the correct mirrored points.
        match sketch.entities.get(result.mirrored_entities[2]).unwrap() {
            SketchEntity::Line { start, end } => {
                assert_eq!(*start, result.mirrored_entities[0]);
                assert_eq!(*end, result.mirrored_entities[1]);
            }
            _ => panic!("Expected mirrored Line entity"),
        }
    }

    #[test]
    fn mirror_arc_reverses_direction() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();

        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });
        let arc_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(6.0, 5.0),
        });
        let arc_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 6.0),
        });
        let arc = sketch.add_entity(SketchEntity::Arc {
            center,
            start: arc_start,
            end: arc_end,
        });

        let result = mirror_entities(
            &mut sketch,
            &[center, arc_start, arc_end, arc],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 4);

        // Check mirrored positions.
        let mc = get_point_position(&sketch, result.mirrored_entities[0]);
        let ms = get_point_position(&sketch, result.mirrored_entities[1]);
        let me = get_point_position(&sketch, result.mirrored_entities[2]);
        assert_pt_eq(&mc, &Pt2::new(-5.0, 5.0));
        assert_pt_eq(&ms, &Pt2::new(-6.0, 5.0));
        assert_pt_eq(&me, &Pt2::new(-5.0, 6.0));

        // Arc direction should be reversed: mirrored arc's start = mirror of original end,
        // mirrored arc's end = mirror of original start.
        match sketch.entities.get(result.mirrored_entities[3]).unwrap() {
            SketchEntity::Arc { center: c, start: s, end: e } => {
                assert_eq!(*c, result.mirrored_entities[0]); // center
                assert_eq!(*s, result.mirrored_entities[2]); // was arc_end's mirror
                assert_eq!(*e, result.mirrored_entities[1]); // was arc_start's mirror
            }
            _ => panic!("Expected mirrored Arc entity"),
        }
    }

    #[test]
    fn mirror_circle() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();

        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });
        let circle = sketch.add_entity(SketchEntity::Circle {
            center,
            radius: 3.0,
        });

        let result = mirror_entities(
            &mut sketch,
            &[center, circle],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 2);
        let mc = get_point_position(&sketch, result.mirrored_entities[0]);
        assert_pt_eq(&mc, &Pt2::new(-5.0, 5.0));

        match sketch.entities.get(result.mirrored_entities[1]).unwrap() {
            SketchEntity::Circle { center: c, radius } => {
                assert_eq!(*c, result.mirrored_entities[0]);
                assert!(approx_eq(*radius, 3.0));
            }
            _ => panic!("Expected mirrored Circle entity"),
        }
    }

    #[test]
    fn mirror_with_constraints() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();

        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 3.0),
        });

        let result = mirror_entities(
            &mut sketch,
            &[pt],
            &MirrorLine::Entity(line_id),
            true,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 1);
        assert_eq!(result.constraints.len(), 1);

        let constraint = sketch.constraints.get(result.constraints[0]).unwrap();
        match &constraint.kind {
            ConstraintKind::Symmetric { axis } => {
                assert_eq!(*axis, line_id);
            }
            _ => panic!("Expected Symmetric constraint"),
        }
        assert_eq!(constraint.entities.len(), 2);
        assert_eq!(constraint.entities[0], pt);
        assert_eq!(constraint.entities[1], result.mirrored_entities[0]);
    }

    #[test]
    fn mirror_about_two_points() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 2.0),
        });

        // Mirror about x-axis defined by two points.
        let result = mirror_entities(
            &mut sketch,
            &[pt],
            &MirrorLine::TwoPoints(Pt2::new(0.0, 0.0), Pt2::new(10.0, 0.0)),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 1);
        let pos = get_point_position(&sketch, result.mirrored_entities[0]);
        assert_pt_eq(&pos, &Pt2::new(3.0, -2.0));
    }

    #[test]
    fn mirror_two_points_no_constraints_without_axis_entity() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 2.0),
        });

        // Even with add_constraints=true, TwoPoints has no axis entity so no constraints.
        let result = mirror_entities(
            &mut sketch,
            &[pt],
            &MirrorLine::TwoPoints(Pt2::new(0.0, 0.0), Pt2::new(10.0, 0.0)),
            true,
        )
        .unwrap();

        assert!(result.constraints.is_empty());
    }

    #[test]
    fn mirror_line_with_implicit_points() {
        // Test mirroring only a Line entity (its point references are not in entity_ids).
        let (mut sketch, line_id) = make_sketch_with_mirror_line();

        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(3.0, 1.0),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 4.0),
        });
        let user_line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        // Only pass the line, not its points.
        let result = mirror_entities(
            &mut sketch,
            &[user_line],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 1);

        // The mirrored line should reference newly created mirrored points.
        match sketch.entities.get(result.mirrored_entities[0]).unwrap() {
            SketchEntity::Line { start, end } => {
                let sp = get_point_position(&sketch, *start);
                let ep = get_point_position(&sketch, *end);
                assert_pt_eq(&sp, &Pt2::new(-3.0, 1.0));
                assert_pt_eq(&ep, &Pt2::new(-5.0, 4.0));
            }
            _ => panic!("Expected mirrored Line entity"),
        }
    }

    #[test]
    fn mirror_empty_list() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();
        let result = mirror_entities(
            &mut sketch,
            &[],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();
        assert!(result.mirrored_entities.is_empty());
        assert!(result.constraints.is_empty());
    }

    #[test]
    fn mirror_degenerate_line_returns_error() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let pt = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 1.0),
        });
        let result = mirror_entities(
            &mut sketch,
            &[pt],
            &MirrorLine::TwoPoints(Pt2::new(0.0, 0.0), Pt2::new(0.0, 0.0)),
            false,
        );
        assert!(result.is_err());
    }

    #[test]
    fn mirror_spline() {
        let (mut sketch, line_id) = make_sketch_with_mirror_line();

        let cp0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(1.0, 0.0),
        });
        let cp1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(2.0, 3.0),
        });
        let cp2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(4.0, 1.0),
        });
        let spline = sketch.add_entity(SketchEntity::Spline {
            control_points: vec![cp0, cp1, cp2],
            degree: 2,
        });

        let result = mirror_entities(
            &mut sketch,
            &[cp0, cp1, cp2, spline],
            &MirrorLine::Entity(line_id),
            false,
        )
        .unwrap();

        assert_eq!(result.mirrored_entities.len(), 4);

        // Verify control point positions.
        let mp0 = get_point_position(&sketch, result.mirrored_entities[0]);
        let mp1 = get_point_position(&sketch, result.mirrored_entities[1]);
        let mp2 = get_point_position(&sketch, result.mirrored_entities[2]);
        assert_pt_eq(&mp0, &Pt2::new(-1.0, 0.0));
        assert_pt_eq(&mp1, &Pt2::new(-2.0, 3.0));
        assert_pt_eq(&mp2, &Pt2::new(-4.0, 1.0));

        // Verify spline references.
        match sketch.entities.get(result.mirrored_entities[3]).unwrap() {
            SketchEntity::Spline {
                control_points,
                degree,
            } => {
                assert_eq!(*degree, 2);
                assert_eq!(control_points.len(), 3);
                assert_eq!(control_points[0], result.mirrored_entities[0]);
                assert_eq!(control_points[1], result.mirrored_entities[1]);
                assert_eq!(control_points[2], result.mirrored_entities[2]);
            }
            _ => panic!("Expected mirrored Spline entity"),
        }
    }
}
