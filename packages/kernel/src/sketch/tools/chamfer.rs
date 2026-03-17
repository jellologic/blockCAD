//! Sketch chamfer tool: bevels corners between intersecting sketch entities
//! by inserting a straight line segment.

use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt2;
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Chamfer sizing mode.
#[derive(Debug, Clone)]
pub enum ChamferMode {
    /// Same distance on both entities from the corner.
    EqualDistance(f64),
    /// Different distance on each entity (d1 on first found entity, d2 on second).
    TwoDistance(f64, f64),
    /// Distance on first entity + chamfer angle in radians.
    DistanceAngle(f64, f64),
}

/// Result of a sketch chamfer operation.
#[derive(Debug)]
pub struct SketchChamferResult {
    /// The new line entity forming the chamfer.
    pub chamfer_line: SketchEntityId,
    /// The entities that were trimmed.
    pub modified_entities: Vec<SketchEntityId>,
}

/// Information about a sketch entity meeting at a vertex, along with
/// which endpoint touches the vertex.
struct EntityAtVertex {
    id: SketchEntityId,
    /// True if the entity's *start* point is at the vertex; false if its *end* point is.
    vertex_is_start: bool,
}

/// Resolve the position of a point entity.
fn point_position(sketch: &Sketch, pid: SketchEntityId) -> KernelResult<Pt2> {
    match sketch.entities.get(pid)? {
        SketchEntity::Point { position } => Ok(*position),
        _ => Err(KernelError::Geometry(
            "Expected a Point entity".into(),
        )),
    }
}

/// Find the two line/arc entities whose start or end point matches `vertex_point`
/// within `tolerance`. Returns exactly two entities or an error.
fn find_entities_at_vertex(
    sketch: &Sketch,
    vertex_point: Pt2,
    tolerance: f64,
) -> KernelResult<(EntityAtVertex, EntityAtVertex)> {
    let mut found: Vec<EntityAtVertex> = Vec::new();

    for (eid, entity) in sketch.entities.iter() {
        match entity {
            SketchEntity::Line { start, end } => {
                let sp = point_position(sketch, *start)?;
                let ep = point_position(sketch, *end)?;
                if (sp - vertex_point).norm() < tolerance {
                    found.push(EntityAtVertex { id: eid, vertex_is_start: true });
                } else if (ep - vertex_point).norm() < tolerance {
                    found.push(EntityAtVertex { id: eid, vertex_is_start: false });
                }
            }
            SketchEntity::Arc { center: _, start, end } => {
                let sp = point_position(sketch, *start)?;
                let ep = point_position(sketch, *end)?;
                if (sp - vertex_point).norm() < tolerance {
                    found.push(EntityAtVertex { id: eid, vertex_is_start: true });
                } else if (ep - vertex_point).norm() < tolerance {
                    found.push(EntityAtVertex { id: eid, vertex_is_start: false });
                }
            }
            _ => {}
        }
    }

    if found.len() < 2 {
        return Err(KernelError::Operation {
            op: "sketch_chamfer".into(),
            detail: format!(
                "Expected 2 entities at vertex ({}, {}), found {}",
                vertex_point.x, vertex_point.y, found.len()
            ),
        });
    }
    if found.len() > 2 {
        return Err(KernelError::Operation {
            op: "sketch_chamfer".into(),
            detail: format!(
                "Ambiguous vertex: {} entities meet at ({}, {})",
                found.len(), vertex_point.x, vertex_point.y
            ),
        });
    }

    let second = found.pop().unwrap();
    let first = found.pop().unwrap();
    Ok((first, second))
}

/// Compute a point along a line entity at distance `d` from the vertex end.
/// Returns the new point position.
fn line_chamfer_point(
    sketch: &Sketch,
    entity: &SketchEntity,
    vertex_is_start: bool,
    d: f64,
) -> KernelResult<Pt2> {
    let (start_id, end_id) = match entity {
        SketchEntity::Line { start, end } => (*start, *end),
        _ => return Err(KernelError::Geometry("Expected Line entity".into())),
    };
    let sp = point_position(sketch, start_id)?;
    let ep = point_position(sketch, end_id)?;

    let (from, to) = if vertex_is_start { (sp, ep) } else { (ep, sp) };
    let dir = to - from;
    let length = dir.norm();
    if d >= length {
        return Err(KernelError::InvalidParameter {
            param: "chamfer distance".into(),
            value: format!("{} (entity length is {})", d, length),
        });
    }
    // Point at distance d from `from` towards `to`
    Ok(from + dir.normalize() * d)
}

/// Compute a point along an arc entity at distance `d` (arc-length) from the vertex end.
fn arc_chamfer_point(
    sketch: &Sketch,
    entity: &SketchEntity,
    vertex_is_start: bool,
    d: f64,
) -> KernelResult<Pt2> {
    let (center_id, start_id, end_id) = match entity {
        SketchEntity::Arc { center, start, end } => (*center, *start, *end),
        _ => return Err(KernelError::Geometry("Expected Arc entity".into())),
    };

    let center = point_position(sketch, center_id)?;
    let sp = point_position(sketch, start_id)?;
    let ep = point_position(sketch, end_id)?;

    let radius = (sp - center).norm();
    if radius < 1e-15 {
        return Err(KernelError::Geometry("Degenerate arc (zero radius)".into()));
    }

    // Compute angles
    let start_angle = (sp.y - center.y).atan2(sp.x - center.x);
    let end_angle = (ep.y - center.y).atan2(ep.x - center.x);

    // Sweep angle (CCW positive)
    let mut sweep = end_angle - start_angle;
    if sweep <= 0.0 {
        sweep += 2.0 * std::f64::consts::PI;
    }

    let arc_length = radius * sweep;
    if d >= arc_length {
        return Err(KernelError::InvalidParameter {
            param: "chamfer distance".into(),
            value: format!("{} (arc length is {})", d, arc_length),
        });
    }

    // Angular offset corresponding to arc-length d
    let d_angle = d / radius;

    let angle = if vertex_is_start {
        // Move from start towards end
        start_angle + d_angle
    } else {
        // Move from end towards start
        end_angle - d_angle
    };

    Ok(Pt2::new(
        center.x + radius * angle.cos(),
        center.y + radius * angle.sin(),
    ))
}

/// Compute the chamfer point on an entity at the given distance from the vertex.
fn entity_chamfer_point(
    sketch: &Sketch,
    eid: SketchEntityId,
    vertex_is_start: bool,
    d: f64,
) -> KernelResult<Pt2> {
    let entity = sketch.entities.get(eid)?;
    match entity {
        SketchEntity::Line { .. } => line_chamfer_point(sketch, entity, vertex_is_start, d),
        SketchEntity::Arc { .. } => arc_chamfer_point(sketch, entity, vertex_is_start, d),
        _ => Err(KernelError::Operation {
            op: "sketch_chamfer".into(),
            detail: "Chamfer is only supported on Line and Arc entities".into(),
        }),
    }
}

/// Apply a chamfer at a vertex (intersection point) in a sketch.
///
/// The function finds the two entities meeting at `vertex_point`, calculates
/// the chamfer trim points, trims the original entities, and inserts a new
/// line segment connecting the chamfer points.
pub fn sketch_chamfer(
    sketch: &mut Sketch,
    vertex_point: Pt2,
    mode: ChamferMode,
    tolerance: f64,
) -> KernelResult<SketchChamferResult> {
    // Validate distances
    match &mode {
        ChamferMode::EqualDistance(d) => {
            if *d <= 0.0 {
                return Err(KernelError::InvalidParameter {
                    param: "chamfer distance".into(),
                    value: format!("{}", d),
                });
            }
        }
        ChamferMode::TwoDistance(d1, d2) => {
            if *d1 <= 0.0 || *d2 <= 0.0 {
                return Err(KernelError::InvalidParameter {
                    param: "chamfer distances".into(),
                    value: format!("({}, {})", d1, d2),
                });
            }
        }
        ChamferMode::DistanceAngle(d, a) => {
            if *d <= 0.0 {
                return Err(KernelError::InvalidParameter {
                    param: "chamfer distance".into(),
                    value: format!("{}", d),
                });
            }
            if *a <= 0.0 || *a >= std::f64::consts::FRAC_PI_2 {
                return Err(KernelError::InvalidParameter {
                    param: "chamfer angle".into(),
                    value: format!("{} (must be in (0, pi/2))", a),
                });
            }
        }
    }

    // Find the two entities at the vertex
    let (ent_a, ent_b) = find_entities_at_vertex(sketch, vertex_point, tolerance)?;

    // Compute distances for each entity
    let (d_a, d_b) = match &mode {
        ChamferMode::EqualDistance(d) => (*d, *d),
        ChamferMode::TwoDistance(d1, d2) => (*d1, *d2),
        ChamferMode::DistanceAngle(d, angle) => {
            // d is distance on entity A; distance on entity B is d * tan(angle)
            (*d, *d * angle.tan())
        }
    };

    // Compute chamfer points
    let pt_a = entity_chamfer_point(sketch, ent_a.id, ent_a.vertex_is_start, d_a)?;
    let pt_b = entity_chamfer_point(sketch, ent_b.id, ent_b.vertex_is_start, d_b)?;

    // Insert new point entities for the chamfer endpoints
    let chamfer_pt_a_id = sketch.add_entity(SketchEntity::Point { position: pt_a });
    let chamfer_pt_b_id = sketch.add_entity(SketchEntity::Point { position: pt_b });

    // Trim entity A: move the vertex-side endpoint to the chamfer point
    trim_entity(sketch, ent_a.id, ent_a.vertex_is_start, chamfer_pt_a_id)?;

    // Trim entity B: move the vertex-side endpoint to the chamfer point
    trim_entity(sketch, ent_b.id, ent_b.vertex_is_start, chamfer_pt_b_id)?;

    // Insert the chamfer line
    let chamfer_line_id = sketch.add_entity(SketchEntity::Line {
        start: chamfer_pt_a_id,
        end: chamfer_pt_b_id,
    });

    Ok(SketchChamferResult {
        chamfer_line: chamfer_line_id,
        modified_entities: vec![ent_a.id, ent_b.id],
    })
}

/// Trim an entity by replacing the endpoint at the vertex side with a new point.
fn trim_entity(
    sketch: &mut Sketch,
    eid: SketchEntityId,
    vertex_is_start: bool,
    new_point_id: SketchEntityId,
) -> KernelResult<()> {
    let entity = sketch.entities.get_mut(eid)?;
    match entity {
        SketchEntity::Line { start, end } => {
            if vertex_is_start {
                *start = new_point_id;
            } else {
                *end = new_point_id;
            }
        }
        SketchEntity::Arc { start, end, .. } => {
            if vertex_is_start {
                *start = new_point_id;
            } else {
                *end = new_point_id;
            }
        }
        _ => {
            return Err(KernelError::Operation {
                op: "trim_entity".into(),
                detail: "Can only trim Line or Arc entities".into(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    /// Helper: create a sketch with two perpendicular lines meeting at the origin.
    /// Line 1: (0,0) -> (10,0)   (along +X)
    /// Line 2: (0,0) -> (0,10)   (along +Y)
    /// Returns (sketch, line1_id, line2_id, shared_point_id)
    fn perpendicular_lines_sketch() -> (Sketch, SketchEntityId, SketchEntityId, SketchEntityId) {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let origin = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let px = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let py = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 10.0),
        });
        let line1 = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: px,
        });
        let line2 = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: py,
        });
        (sketch, line1, line2, origin)
    }

    #[test]
    fn test_equal_distance_chamfer_perpendicular_lines() {
        let (mut sketch, _line1, _line2, _origin) = perpendicular_lines_sketch();
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::EqualDistance(3.0),
            1e-6,
        )
        .expect("chamfer should succeed");

        // Two entities should be modified
        assert_eq!(result.modified_entities.len(), 2);

        // The chamfer line should exist
        let chamfer = sketch.entities.get(result.chamfer_line).unwrap();
        match chamfer {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                // One chamfer point should be at (3,0), the other at (0,3)
                let pts = vec![sp, ep];
                let expected_a = Pt2::new(3.0, 0.0);
                let expected_b = Pt2::new(0.0, 3.0);
                let has_a = pts.iter().any(|p| (p - expected_a).norm() < 1e-9);
                let has_b = pts.iter().any(|p| (p - expected_b).norm() < 1e-9);
                assert!(has_a, "chamfer should have point near (3,0), got {:?}", pts);
                assert!(has_b, "chamfer should have point near (0,3), got {:?}", pts);
            }
            _ => panic!("chamfer_line should be a Line entity"),
        }
    }

    #[test]
    fn test_two_distance_chamfer() {
        let (mut sketch, _line1, _line2, _origin) = perpendicular_lines_sketch();
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::TwoDistance(2.0, 5.0),
            1e-6,
        )
        .expect("chamfer should succeed");

        let chamfer = sketch.entities.get(result.chamfer_line).unwrap();
        match chamfer {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                let pts = vec![sp, ep];
                // d1=2 on first entity (line along X), d2=5 on second entity (line along Y)
                let expected_a = Pt2::new(2.0, 0.0);
                let expected_b = Pt2::new(0.0, 5.0);
                let has_a = pts.iter().any(|p| (p - expected_a).norm() < 1e-9);
                let has_b = pts.iter().any(|p| (p - expected_b).norm() < 1e-9);
                assert!(has_a, "chamfer should have point near (2,0), got {:?}", pts);
                assert!(has_b, "chamfer should have point near (0,5), got {:?}", pts);
            }
            _ => panic!("chamfer_line should be a Line entity"),
        }
    }

    #[test]
    fn test_distance_angle_chamfer() {
        let (mut sketch, _line1, _line2, _origin) = perpendicular_lines_sketch();
        let angle = std::f64::consts::FRAC_PI_4; // 45 degrees => d2 = d1 * tan(45) = d1
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::DistanceAngle(4.0, angle),
            1e-6,
        )
        .expect("chamfer should succeed");

        let chamfer = sketch.entities.get(result.chamfer_line).unwrap();
        match chamfer {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                let pts = vec![sp, ep];
                // d=4 on first entity, angle=45deg => d2 = 4*tan(45) = 4
                let expected_a = Pt2::new(4.0, 0.0);
                let expected_b = Pt2::new(0.0, 4.0);
                let has_a = pts.iter().any(|p| (p - expected_a).norm() < 1e-9);
                let has_b = pts.iter().any(|p| (p - expected_b).norm() < 1e-9);
                assert!(has_a, "chamfer should have point near (4,0), got {:?}", pts);
                assert!(has_b, "chamfer should have point near (0,4), got {:?}", pts);
            }
            _ => panic!("chamfer_line should be a Line entity"),
        }
    }

    #[test]
    fn test_chamfer_distance_too_large() {
        let (mut sketch, _line1, _line2, _origin) = perpendicular_lines_sketch();
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::EqualDistance(15.0), // lines are only length 10
            1e-6,
        );
        assert!(result.is_err(), "should fail when distance exceeds entity length");
        match result.unwrap_err() {
            KernelError::InvalidParameter { param, .. } => {
                assert!(param.contains("chamfer distance"));
            }
            other => panic!("expected InvalidParameter, got {:?}", other),
        }
    }

    #[test]
    fn test_chamfer_on_line_arc_intersection() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Arc: center at (0,0), from (5,0) going CCW to (0,5) -- quarter circle, radius 5
        let center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let arc_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        let arc_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 5.0),
        });
        let _arc = sketch.add_entity(SketchEntity::Arc {
            center,
            start: arc_start,
            end: arc_end,
        });

        // Line from (0,5) going up to (0,15)
        let line_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 15.0),
        });
        let _line = sketch.add_entity(SketchEntity::Line {
            start: arc_end, // shares the arc's end point
            end: line_end,
        });

        // Chamfer at (0,5) where the arc end meets the line start
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 5.0),
            ChamferMode::EqualDistance(2.0),
            1e-6,
        )
        .expect("chamfer on line-arc intersection should succeed");

        assert_eq!(result.modified_entities.len(), 2);

        // Verify the chamfer line exists and connects two valid points
        let chamfer = sketch.entities.get(result.chamfer_line).unwrap();
        match chamfer {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                // The line chamfer point should be at (0, 5+2) = (0,7) since the line
                // goes from (0,5) to (0,15) and we move 2 along it from the start.
                // But after chamfer the line's start is moved, so let's check the
                // chamfer line endpoints instead.
                let chamfer_len = (sp - ep).norm();
                assert!(
                    chamfer_len > 0.0,
                    "chamfer line should have non-zero length"
                );

                // One endpoint should be on the line (x=0, y between 5 and 15)
                let on_line = [sp, ep]
                    .iter()
                    .any(|p| p.x.abs() < 1e-6 && p.y > 5.0 - 1e-6);
                assert!(on_line, "one chamfer endpoint should lie on the line");

                // One endpoint should be on the arc (distance ~5 from origin)
                let on_arc = [sp, ep]
                    .iter()
                    .any(|p| ((p - Pt2::new(0.0, 0.0)).norm() - 5.0).abs() < 0.1);
                assert!(on_arc, "one chamfer endpoint should lie on the arc");
            }
            _ => panic!("chamfer_line should be a Line entity"),
        }
    }

    #[test]
    fn test_chamfer_no_entities_at_vertex() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(99.0, 99.0),
            ChamferMode::EqualDistance(1.0),
            1e-6,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_chamfer_negative_distance_rejected() {
        let (mut sketch, _, _, _) = perpendicular_lines_sketch();
        let result = sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::EqualDistance(-1.0),
            1e-6,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_chamfer_entities_are_trimmed() {
        let (mut sketch, line1, line2, _origin) = perpendicular_lines_sketch();
        sketch_chamfer(
            &mut sketch,
            Pt2::new(0.0, 0.0),
            ChamferMode::EqualDistance(3.0),
            1e-6,
        )
        .expect("chamfer should succeed");

        // After chamfer, line1 should no longer start at the origin.
        // Its start should now be the chamfer point at (3,0).
        let l1 = sketch.entities.get(line1).unwrap();
        match l1 {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                // The start was at origin, so it should have been moved to (3,0)
                assert!(
                    (sp - Pt2::new(3.0, 0.0)).norm() < 1e-9,
                    "line1 start should be trimmed to (3,0), got ({}, {})",
                    sp.x, sp.y
                );
                // The far end should be unchanged
                assert!(
                    (ep - Pt2::new(10.0, 0.0)).norm() < 1e-9,
                    "line1 end should remain at (10,0)"
                );
            }
            _ => panic!("line1 should still be a Line"),
        }

        let l2 = sketch.entities.get(line2).unwrap();
        match l2 {
            SketchEntity::Line { start, end } => {
                let sp = point_position(&sketch, *start).unwrap();
                let ep = point_position(&sketch, *end).unwrap();
                assert!(
                    (sp - Pt2::new(0.0, 3.0)).norm() < 1e-9,
                    "line2 start should be trimmed to (0,3), got ({}, {})",
                    sp.x, sp.y
                );
                assert!(
                    (ep - Pt2::new(0.0, 10.0)).norm() < 1e-9,
                    "line2 end should remain at (0,10)"
                );
            }
            _ => panic!("line2 should still be a Line"),
        }
    }
}
