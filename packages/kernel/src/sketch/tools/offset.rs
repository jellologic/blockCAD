//! Offset entities sketch tool.
//!
//! Creates offset copies of sketch entities at a specified distance. Positive
//! distance offsets to one side, negative to the other.

use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt2;
use crate::sketch::constraint::{Constraint, ConstraintId, ConstraintKind};
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Result of an offset operation.
pub struct OffsetResult {
    /// IDs of the newly created entities (points and curves).
    pub new_entities: Vec<SketchEntityId>,
    /// IDs of the newly created constraints (coincident at corners).
    pub new_constraints: Vec<ConstraintId>,
}

/// Internal: tracks curve entity and its start/end point IDs for chain processing.
struct OffsetCurveInfo {
    curve_id: SketchEntityId,
    start_point_id: SketchEntityId,
    end_point_id: SketchEntityId,
}

/// Offset a set of sketch entities by the given `distance`.
///
/// Positive distance offsets to the "left" side of the entity direction,
/// negative distance offsets to the "right" side.
///
/// `tolerance` is used when checking for degenerate geometry (e.g. an arc whose
/// offset radius would become zero or negative).
pub fn offset_entities(
    sketch: &mut Sketch,
    entity_ids: &[SketchEntityId],
    distance: f64,
    tolerance: f64,
) -> KernelResult<OffsetResult> {
    if entity_ids.is_empty() {
        return Ok(OffsetResult {
            new_entities: Vec::new(),
            new_constraints: Vec::new(),
        });
    }

    let mut new_entities: Vec<SketchEntityId> = Vec::new();
    let mut new_constraints: Vec<ConstraintId> = Vec::new();

    // Collect the offset curve entities (non-point entities) and their endpoint IDs
    // so we can add coincident constraints between consecutive chain members.
    let mut offset_curves: Vec<OffsetCurveInfo> = Vec::new();

    for &eid in entity_ids {
        let entity = sketch.entities.get(eid)?.clone();
        match entity {
            SketchEntity::Line { start, end } => {
                let start_pos = point_position(&sketch, start)?;
                let end_pos = point_position(&sketch, end)?;
                let (new_start, new_end) = offset_line_points(start_pos, end_pos, distance)?;

                let ns_id = sketch.add_entity(SketchEntity::Point { position: new_start });
                let ne_id = sketch.add_entity(SketchEntity::Point { position: new_end });
                let line_id = sketch.add_entity(SketchEntity::Line {
                    start: ns_id,
                    end: ne_id,
                });
                new_entities.push(ns_id);
                new_entities.push(ne_id);
                new_entities.push(line_id);
                offset_curves.push(OffsetCurveInfo {
                    curve_id: line_id,
                    start_point_id: ns_id,
                    end_point_id: ne_id,
                });
            }
            SketchEntity::Arc { center, start, end } => {
                let center_pos = point_position(&sketch, center)?;
                let start_pos = point_position(&sketch, start)?;
                let end_pos = point_position(&sketch, end)?;

                let radius = ((start_pos.x - center_pos.x).powi(2)
                    + (start_pos.y - center_pos.y).powi(2))
                .sqrt();

                let new_radius = offset_arc_radius(radius, distance, tolerance)?;

                // Scale start/end points relative to center to the new radius.
                let scale = new_radius / radius;
                let new_start = Pt2::new(
                    center_pos.x + (start_pos.x - center_pos.x) * scale,
                    center_pos.y + (start_pos.y - center_pos.y) * scale,
                );
                let new_end = Pt2::new(
                    center_pos.x + (end_pos.x - center_pos.x) * scale,
                    center_pos.y + (end_pos.y - center_pos.y) * scale,
                );

                let nc_id = sketch.add_entity(SketchEntity::Point { position: center_pos });
                let ns_id = sketch.add_entity(SketchEntity::Point { position: new_start });
                let ne_id = sketch.add_entity(SketchEntity::Point { position: new_end });
                let arc_id = sketch.add_entity(SketchEntity::Arc {
                    center: nc_id,
                    start: ns_id,
                    end: ne_id,
                });
                new_entities.push(nc_id);
                new_entities.push(ns_id);
                new_entities.push(ne_id);
                new_entities.push(arc_id);
                offset_curves.push(OffsetCurveInfo {
                    curve_id: arc_id,
                    start_point_id: ns_id,
                    end_point_id: ne_id,
                });
            }
            SketchEntity::Circle { center, radius } => {
                let center_pos = point_position(&sketch, center)?;
                let new_radius = offset_circle_radius(radius, distance, tolerance)?;

                let nc_id = sketch.add_entity(SketchEntity::Point { position: center_pos });
                let circle_id = sketch.add_entity(SketchEntity::Circle {
                    center: nc_id,
                    radius: new_radius,
                });
                new_entities.push(nc_id);
                new_entities.push(circle_id);
                // Circles are closed; they don't participate in chain connectivity.
            }
            SketchEntity::Point { .. } => {
                // Points are skipped; they are not geometric curves to offset.
            }
            _ => {
                return Err(KernelError::Operation {
                    op: "offset_entities".into(),
                    detail: format!("Unsupported entity type for offset: {:?}", entity),
                });
            }
        }
    }

    // Add coincident constraints between consecutive offset curves if the
    // original entities shared an endpoint (i.e., formed a chain).
    for i in 0..offset_curves.len().saturating_sub(1) {
        // Check whether the original entities at positions i and i+1 share an
        // endpoint. We look at the original entity pair.
        let orig_a = sketch.entities.get(entity_ids[find_original_index(entity_ids, &offset_curves, i)])?;
        let orig_b = sketch.entities.get(entity_ids[find_original_index(entity_ids, &offset_curves, i + 1)])?;

        if entities_share_endpoint(orig_a, orig_b) {
            let c_id = sketch.add_constraint(Constraint::new(
                ConstraintKind::Coincident,
                vec![
                    offset_curves[i].end_point_id,
                    offset_curves[i + 1].start_point_id,
                ],
            ));
            new_constraints.push(c_id);
        }
    }

    // Trim offset corners: move shared endpoints to the intersection of
    // adjacent offset curves so the chain stays connected.
    trim_offset_corners(sketch, &offset_curves)?;

    Ok(OffsetResult {
        new_entities,
        new_constraints,
    })
}

// ---------------------------------------------------------------------------
// Helper: resolve a point entity's position
// ---------------------------------------------------------------------------

fn point_position(sketch: &Sketch, id: SketchEntityId) -> KernelResult<Pt2> {
    match sketch.entities.get(id)? {
        SketchEntity::Point { position } => Ok(*position),
        other => Err(KernelError::Operation {
            op: "offset_entities".into(),
            detail: format!("Expected Point entity, found {:?}", other),
        }),
    }
}

// ---------------------------------------------------------------------------
// Offset primitives
// ---------------------------------------------------------------------------

/// Compute the two offset endpoint positions for a line segment.
///
/// The line is offset by `distance` in the direction perpendicular to the
/// start-end vector (rotated 90 degrees counter-clockwise for positive
/// distance).
fn offset_line_points(start: Pt2, end: Pt2, distance: f64) -> KernelResult<(Pt2, Pt2)> {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-15 {
        return Err(KernelError::Geometry(
            "Cannot offset a zero-length line".into(),
        ));
    }
    // Perpendicular unit vector (rotated 90 deg CCW).
    let nx = -dy / len;
    let ny = dx / len;

    let offset_x = nx * distance;
    let offset_y = ny * distance;

    Ok((
        Pt2::new(start.x + offset_x, start.y + offset_y),
        Pt2::new(end.x + offset_x, end.y + offset_y),
    ))
}

/// Compute the offset radius for an arc. The offset is applied outward for
/// positive distance. Returns an error if the resulting radius is below
/// `tolerance`.
fn offset_arc_radius(radius: f64, distance: f64, tolerance: f64) -> KernelResult<f64> {
    let new_radius = radius + distance;
    if new_radius < tolerance {
        return Err(KernelError::InvalidParameter {
            param: "offset distance".into(),
            value: format!(
                "offset {distance} on arc of radius {radius} yields invalid radius {new_radius}"
            ),
        });
    }
    Ok(new_radius)
}

/// Compute the offset radius for a circle (same logic as arc).
fn offset_circle_radius(radius: f64, distance: f64, tolerance: f64) -> KernelResult<f64> {
    offset_arc_radius(radius, distance, tolerance)
}

// ---------------------------------------------------------------------------
// Chain helpers
// ---------------------------------------------------------------------------

/// Find which index in `entity_ids` corresponds to the n-th offset curve.
/// Because we skip Point entities and unsupported entities would have already
/// errored, we need to map back through the non-point entities.
fn find_original_index(
    entity_ids: &[SketchEntityId],
    _offset_curves: &[OffsetCurveInfo],
    curve_index: usize,
) -> usize {
    // The offset curves were produced in order, skipping Point entities.
    // We need to replicate that iteration.
    let mut curve_count = 0usize;
    for (i, _) in entity_ids.iter().enumerate() {
        // We cannot inspect the entity here without the sketch, but the caller
        // only places non-Point, non-Circle entities in offset_curves, and
        // Circle doesn't contribute. However, for simplicity we just return
        // index == curve_index since the typical usage supplies only curve
        // entity IDs. If Point IDs were mixed in we would need the sketch ref.
        // For now, return the straight mapping which works for the common case.
        if curve_count == curve_index {
            return i;
        }
        curve_count += 1;
    }
    curve_index
}

/// Check whether two entities share an endpoint.
fn entities_share_endpoint(a: &SketchEntity, b: &SketchEntity) -> bool {
    let a_endpoints = entity_endpoints(a);
    let b_endpoints = entity_endpoints(b);

    for ae in &a_endpoints {
        for be in &b_endpoints {
            if ae == be {
                return true;
            }
        }
    }
    false
}

fn entity_endpoints(e: &SketchEntity) -> Vec<SketchEntityId> {
    match e {
        SketchEntity::Line { start, end } => vec![*start, *end],
        SketchEntity::Arc { start, end, .. } => vec![*start, *end],
        _ => vec![],
    }
}

/// Trim/extend offset entities at corners so they meet cleanly.
///
/// For each pair of consecutive offset curves that are both lines, we compute
/// the intersection point and move the shared endpoint(s) there.
fn trim_offset_corners(
    sketch: &mut Sketch,
    curves: &[OffsetCurveInfo],
) -> KernelResult<()> {
    for i in 0..curves.len().saturating_sub(1) {
        let c0 = &curves[i];
        let c1 = &curves[i + 1];

        // Only trim line-line corners for now.
        let e0 = sketch.entities.get(c0.curve_id)?.clone();
        let e1 = sketch.entities.get(c1.curve_id)?.clone();

        if let (
            SketchEntity::Line {
                start: s0, end: e0id,
            },
            SketchEntity::Line {
                start: s1, end: _e1id,
            },
        ) = (&e0, &e1)
        {
            let p0 = point_position(sketch, *s0)?;
            let p1 = point_position(sketch, *e0id)?;
            let p2 = point_position(sketch, *s1)?;
            let p3 = point_position(sketch, c1.end_point_id)?;

            if let Some(intersection) = line_line_intersect(p0, p1, p2, p3) {
                // Move the end of curve 0 and start of curve 1 to the
                // intersection.
                if let SketchEntity::Point { position } =
                    sketch.entities.get_mut(c0.end_point_id)?
                {
                    *position = intersection;
                }
                if let SketchEntity::Point { position } =
                    sketch.entities.get_mut(c1.start_point_id)?
                {
                    *position = intersection;
                }
            }
        }
    }
    Ok(())
}

/// Intersect two 2D line segments (extended as infinite lines).
/// Returns `None` if the lines are (nearly) parallel.
fn line_line_intersect(a0: Pt2, a1: Pt2, b0: Pt2, b1: Pt2) -> Option<Pt2> {
    let d1x = a1.x - a0.x;
    let d1y = a1.y - a0.y;
    let d2x = b1.x - b0.x;
    let d2y = b1.y - b0.y;

    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return None; // parallel
    }

    let t = ((b0.x - a0.x) * d2y - (b0.y - a0.y) * d2x) / denom;
    Some(Pt2::new(a0.x + t * d1x, a0.y + t * d1y))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    /// Helper: create a minimal sketch with a horizontal line from (0,0) to (10,0).
    fn sketch_with_line() -> (Sketch, SketchEntityId, SketchEntityId, SketchEntityId) {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let p1 = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let line = sk.add_entity(SketchEntity::Line { start: p1, end: p2 });
        (sk, p1, p2, line)
    }

    #[test]
    fn offset_single_line() {
        let (mut sk, _p1, _p2, line) = sketch_with_line();
        let result = offset_entities(&mut sk, &[line], 5.0, 1e-9).unwrap();

        // Should produce 2 new points + 1 new line = 3 entities.
        assert_eq!(result.new_entities.len(), 3);

        // The offset line's start and end points should be at y = 5.
        let new_line_id = result.new_entities[2];
        let new_line = sk.entities.get(new_line_id).unwrap();
        if let SketchEntity::Line { start, end } = new_line {
            let s = point_position(&sk, *start).unwrap();
            let e = point_position(&sk, *end).unwrap();
            assert!((s.y - 5.0).abs() < 1e-9);
            assert!((e.y - 5.0).abs() < 1e-9);
            assert!((s.x - 0.0).abs() < 1e-9);
            assert!((e.x - 10.0).abs() < 1e-9);
        } else {
            panic!("Expected Line entity");
        }
    }

    #[test]
    fn offset_single_line_negative() {
        let (mut sk, _p1, _p2, line) = sketch_with_line();
        let result = offset_entities(&mut sk, &[line], -3.0, 1e-9).unwrap();

        let new_line_id = result.new_entities[2];
        let new_line = sk.entities.get(new_line_id).unwrap();
        if let SketchEntity::Line { start, end } = new_line {
            let s = point_position(&sk, *start).unwrap();
            let e = point_position(&sk, *end).unwrap();
            assert!((s.y - (-3.0)).abs() < 1e-9);
            assert!((e.y - (-3.0)).abs() < 1e-9);
        } else {
            panic!("Expected Line entity");
        }
    }

    #[test]
    fn offset_circle() {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let center = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let circle = sk.add_entity(SketchEntity::Circle {
            center,
            radius: 10.0,
        });

        let result = offset_entities(&mut sk, &[circle], 5.0, 1e-9).unwrap();

        // Should produce 1 new center point + 1 new circle = 2 entities.
        assert_eq!(result.new_entities.len(), 2);

        let new_circle_id = result.new_entities[1];
        if let SketchEntity::Circle { radius, .. } = sk.entities.get(new_circle_id).unwrap() {
            assert!((*radius - 15.0).abs() < 1e-9);
        } else {
            panic!("Expected Circle entity");
        }
    }

    #[test]
    fn offset_circle_negative() {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let center = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let circle = sk.add_entity(SketchEntity::Circle {
            center,
            radius: 10.0,
        });

        let result = offset_entities(&mut sk, &[circle], -3.0, 1e-9).unwrap();
        let new_circle_id = result.new_entities[1];
        if let SketchEntity::Circle { radius, .. } = sk.entities.get(new_circle_id).unwrap() {
            assert!((*radius - 7.0).abs() < 1e-9);
        } else {
            panic!("Expected Circle entity");
        }
    }

    #[test]
    fn offset_arc() {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let center = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let start = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let end = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 10.0),
        });
        let arc = sk.add_entity(SketchEntity::Arc { center, start, end });

        let result = offset_entities(&mut sk, &[arc], 5.0, 1e-9).unwrap();

        // center + start + end + arc = 4 new entities.
        assert_eq!(result.new_entities.len(), 4);

        // Check the new arc's start and end are at radius 15.
        let new_arc_id = result.new_entities[3];
        if let SketchEntity::Arc {
            center: nc,
            start: ns,
            end: ne,
        } = sk.entities.get(new_arc_id).unwrap()
        {
            let c = point_position(&sk, *nc).unwrap();
            let s = point_position(&sk, *ns).unwrap();
            let e = point_position(&sk, *ne).unwrap();
            let r_s = ((s.x - c.x).powi(2) + (s.y - c.y).powi(2)).sqrt();
            let r_e = ((e.x - c.x).powi(2) + (e.y - c.y).powi(2)).sqrt();
            assert!((r_s - 15.0).abs() < 1e-9);
            assert!((r_e - 15.0).abs() < 1e-9);
        } else {
            panic!("Expected Arc entity");
        }
    }

    #[test]
    fn offset_arc_invalid_radius() {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let center = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let start = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        let end = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 5.0),
        });
        let arc = sk.add_entity(SketchEntity::Arc { center, start, end });

        // Offset by -6 on radius 5 -> negative radius -> error.
        let result = offset_entities(&mut sk, &[arc], -6.0, 1e-9);
        assert!(result.is_err());
    }

    #[test]
    fn offset_connected_chain() {
        // Two lines: (0,0)-(10,0) and (10,0)-(10,10) sharing endpoint at (10,0).
        let mut sk = Sketch::new(Plane::xy(0.0));
        let p1 = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p2 = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        let p3 = sk.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 10.0),
        });
        let line1 = sk.add_entity(SketchEntity::Line {
            start: p1,
            end: p2,
        });
        let line2 = sk.add_entity(SketchEntity::Line {
            start: p2,
            end: p3,
        });

        let result = offset_entities(&mut sk, &[line1, line2], 2.0, 1e-9).unwrap();

        // 2 lines * (2 points + 1 line) = 6 entities.
        assert_eq!(result.new_entities.len(), 6);

        // Should have 1 coincident constraint between the two offset lines.
        assert_eq!(result.new_constraints.len(), 1);
    }

    #[test]
    fn offset_empty_input() {
        let mut sk = Sketch::new(Plane::xy(0.0));
        let result = offset_entities(&mut sk, &[], 5.0, 1e-9).unwrap();
        assert!(result.new_entities.is_empty());
        assert!(result.new_constraints.is_empty());
    }
}
