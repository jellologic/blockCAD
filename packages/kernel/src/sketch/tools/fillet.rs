//! Sketch fillet tool: rounds corners between intersecting sketch entities
//! by inserting tangent arcs.

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt2, Vec2};
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Result of a sketch fillet operation.
#[derive(Debug, Clone)]
pub struct SketchFilletResult {
    /// The newly created fillet arc entity ID.
    pub fillet_arc: SketchEntityId,
    /// Entities that were modified (trimmed) during the fillet.
    pub modified_entities: Vec<SketchEntityId>,
    /// Entities that were removed during the fillet.
    pub removed_entities: Vec<SketchEntityId>,
}

/// Internal representation of a line for geometric calculations.
#[derive(Debug, Clone)]
struct LineGeom {
    id: SketchEntityId,
    start: Pt2,
    end: Pt2,
}

/// Internal representation of an arc for geometric calculations.
#[derive(Debug, Clone)]
struct ArcGeom {
    id: SketchEntityId,
    center: Pt2,
    start: Pt2,
    #[allow(dead_code)]
    end: Pt2,
    radius: f64,
}

/// An entity adjacent to the fillet vertex.
#[derive(Debug, Clone)]
enum AdjacentEntity {
    Line(LineGeom),
    Arc(ArcGeom),
}

/// Apply a fillet at a vertex point in the sketch.
///
/// Finds the two entities meeting at `vertex_point`, calculates a tangent arc
/// of the given `radius`, trims the original entities to the tangent points,
/// and inserts the fillet arc.
///
/// Supports Line-Line, Line-Arc, and Arc-Arc corners.
pub fn sketch_fillet(
    sketch: &mut Sketch,
    vertex_point: Pt2,
    radius: f64,
    tolerance: f64,
) -> KernelResult<SketchFilletResult> {
    if radius <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "radius".into(),
            value: radius.to_string(),
        });
    }

    // Find the two entities that meet at vertex_point.
    let adjacent = find_adjacent_entities(sketch, vertex_point, tolerance)?;
    if adjacent.len() < 2 {
        return Err(KernelError::Operation {
            op: "sketch_fillet".into(),
            detail: format!(
                "Expected 2 entities at vertex ({}, {}), found {}",
                vertex_point.x,
                vertex_point.y,
                adjacent.len()
            ),
        });
    }

    let entity_a = &adjacent[0];
    let entity_b = &adjacent[1];

    match (entity_a, entity_b) {
        (AdjacentEntity::Line(la), AdjacentEntity::Line(lb)) => {
            fillet_line_line(sketch, la, lb, vertex_point, radius, tolerance)
        }
        (AdjacentEntity::Line(line), AdjacentEntity::Arc(arc))
        | (AdjacentEntity::Arc(arc), AdjacentEntity::Line(line)) => {
            fillet_line_arc(sketch, line, arc, vertex_point, radius, tolerance)
        }
        (AdjacentEntity::Arc(a), AdjacentEntity::Arc(b)) => {
            fillet_arc_arc(sketch, a, b, vertex_point, radius, tolerance)
        }
    }
}

/// Find entities whose start or end point is near `vertex_point`.
fn find_adjacent_entities(
    sketch: &Sketch,
    vertex_point: Pt2,
    tolerance: f64,
) -> KernelResult<Vec<AdjacentEntity>> {
    let mut result = Vec::new();

    for (eid, entity) in sketch.entities.iter() {
        match entity {
            SketchEntity::Line { start, end } => {
                let s = get_point_position(&sketch, *start)?;
                let e = get_point_position(&sketch, *end)?;
                if distance(s, vertex_point) < tolerance || distance(e, vertex_point) < tolerance {
                    result.push(AdjacentEntity::Line(LineGeom {
                        id: eid,
                        start: s,
                        end: e,
                    }));
                }
            }
            SketchEntity::Arc {
                center,
                start,
                end,
            } => {
                let c = get_point_position(&sketch, *center)?;
                let s = get_point_position(&sketch, *start)?;
                let e = get_point_position(&sketch, *end)?;
                let r = distance(c, s);
                if distance(s, vertex_point) < tolerance || distance(e, vertex_point) < tolerance {
                    result.push(AdjacentEntity::Arc(ArcGeom {
                        id: eid,
                        center: c,
                        start: s,
                        end: e,
                        radius: r,
                    }));
                }
            }
            _ => {}
        }
    }

    Ok(result)
}

fn get_point_position(sketch: &Sketch, id: SketchEntityId) -> KernelResult<Pt2> {
    match sketch.entities.get(id)? {
        SketchEntity::Point { position } => Ok(*position),
        _ => Err(KernelError::Operation {
            op: "sketch_fillet".into(),
            detail: "Expected a Point entity".into(),
        }),
    }
}

fn distance(a: Pt2, b: Pt2) -> f64 {
    nalgebra::distance(&a, &b)
}

/// Normalize a Vec2; returns None if the vector is near-zero.
fn try_normalize(v: Vec2) -> Option<Vec2> {
    let len = v.norm();
    if len < 1e-15 {
        None
    } else {
        Some(v / len)
    }
}

/// Compute the foot of perpendicular from `point` onto the infinite line through `a` and `b`.
fn closest_point_on_line(point: Pt2, a: Pt2, b: Pt2) -> Pt2 {
    let ab = b - a;
    let ap = point - a;
    let t = ab.dot(&ap) / ab.dot(&ab);
    a + ab * t
}

/// Fillet between two lines meeting at `vertex`.
fn fillet_line_line(
    sketch: &mut Sketch,
    la: &LineGeom,
    lb: &LineGeom,
    vertex: Pt2,
    radius: f64,
    tolerance: f64,
) -> KernelResult<SketchFilletResult> {
    // Determine directions away from vertex for each line.
    let dir_a = line_direction_from_vertex(la, vertex, tolerance);
    let dir_b = line_direction_from_vertex(lb, vertex, tolerance);

    let dir_a = try_normalize(dir_a).ok_or_else(|| KernelError::Geometry(
        "Degenerate line (zero length) in fillet".into(),
    ))?;
    let dir_b = try_normalize(dir_b).ok_or_else(|| KernelError::Geometry(
        "Degenerate line (zero length) in fillet".into(),
    ))?;

    // The angle bisector direction.
    let bisector = try_normalize(dir_a + dir_b).ok_or_else(|| KernelError::Geometry(
        "Lines are parallel/antiparallel, cannot fillet".into(),
    ))?;

    // Half-angle between the two lines.
    let cos_half = dir_a.dot(&bisector).abs();
    if cos_half < 1e-12 {
        return Err(KernelError::Geometry(
            "Lines are parallel, cannot fillet".into(),
        ));
    }
    let sin_half = (1.0 - cos_half * cos_half).sqrt();
    if sin_half < 1e-12 {
        return Err(KernelError::Geometry(
            "Lines are parallel, cannot fillet".into(),
        ));
    }

    // Distance from vertex to arc center along bisector.
    let center_dist = radius / sin_half;

    // Check that the fillet fits within both lines.
    let tangent_dist = radius / (sin_half / cos_half); // radius / tan(half_angle) = radius * cos/sin
    let len_a = line_available_length(la, vertex, tolerance);
    let len_b = line_available_length(lb, vertex, tolerance);
    if tangent_dist > len_a + tolerance || tangent_dist > len_b + tolerance {
        return Err(KernelError::InvalidParameter {
            param: "radius".into(),
            value: format!(
                "{} (too large for corner; max ~{:.6})",
                radius,
                len_a.min(len_b) * sin_half / cos_half
            ),
        });
    }

    let arc_center = vertex + bisector * center_dist;

    // Tangent points: foot of perpendicular from arc_center onto each line.
    let tp_a = closest_point_on_line(arc_center, la.start, la.end);
    let tp_b = closest_point_on_line(arc_center, lb.start, lb.end);

    // Insert new geometry into the sketch.
    let center_pt = sketch.add_entity(SketchEntity::Point { position: arc_center });
    let arc_start_pt = sketch.add_entity(SketchEntity::Point { position: tp_a });
    let arc_end_pt = sketch.add_entity(SketchEntity::Point { position: tp_b });

    let fillet_arc = sketch.add_entity(SketchEntity::Arc {
        center: center_pt,
        start: arc_start_pt,
        end: arc_end_pt,
    });

    // Trim line A: replace the vertex-side endpoint with the tangent point.
    trim_line_endpoint(sketch, la, vertex, arc_start_pt, tolerance)?;
    // Trim line B: replace the vertex-side endpoint with the tangent point.
    trim_line_endpoint(sketch, lb, vertex, arc_end_pt, tolerance)?;

    Ok(SketchFilletResult {
        fillet_arc,
        modified_entities: vec![la.id, lb.id],
        removed_entities: vec![],
    })
}

/// Fillet between a line and an arc meeting at `vertex`.
fn fillet_line_arc(
    sketch: &mut Sketch,
    line: &LineGeom,
    arc: &ArcGeom,
    vertex: Pt2,
    radius: f64,
    tolerance: f64,
) -> KernelResult<SketchFilletResult> {
    // Direction along line away from vertex.
    let line_dir = try_normalize(line_direction_from_vertex(line, vertex, tolerance))
        .ok_or_else(|| KernelError::Geometry("Degenerate line in fillet".into()))?;

    // Line normal (perpendicular) - choose the side toward the arc center.
    let line_normal_candidate = Vec2::new(-line_dir.y, line_dir.x);
    let to_arc_center = arc.center - vertex;
    let line_normal = if to_arc_center.dot(&line_normal_candidate) > 0.0 {
        line_normal_candidate
    } else {
        -line_normal_candidate
    };

    // Offset line by radius in the normal direction: the fillet center lies on this offset line.
    // The offset line passes through (vertex + line_normal * radius) with direction line_dir.
    let offset_line_pt = vertex + line_normal * radius;

    // The fillet center is also at distance (arc.radius +/- radius) from arc center.
    // If fillet is on the outside: arc.radius + radius
    // If fillet is on the inside: arc.radius - radius (and arc.radius > radius)
    // We try both and pick the one that produces a valid configuration.

    let candidates = [arc.radius + radius, (arc.radius - radius).abs()];
    let mut best_center: Option<Pt2> = None;

    for &d in &candidates {
        if d < 1e-15 {
            continue;
        }
        // Find intersection of offset line with circle of radius d centered at arc.center.
        if let Some(centers) = line_circle_intersection(offset_line_pt, line_dir, arc.center, d) {
            for c in centers {
                // Verify: distance from c to line == radius (within tolerance)
                let foot = closest_point_on_line(c, line.start, line.end);
                let dist_to_line = distance(c, foot);
                if (dist_to_line - radius).abs() < tolerance * 10.0 {
                    // Pick the center closest to the vertex.
                    if best_center.is_none()
                        || distance(c, vertex) < distance(best_center.unwrap(), vertex)
                    {
                        best_center = Some(c);
                    }
                }
            }
        }
    }

    let fillet_center = best_center.ok_or_else(|| KernelError::InvalidParameter {
        param: "radius".into(),
        value: format!("{} (no valid fillet center found for line-arc corner)", radius),
    })?;

    // Tangent point on the line: foot of perpendicular.
    let tp_line = closest_point_on_line(fillet_center, line.start, line.end);

    // Tangent point on the arc: intersection of line(arc.center -> fillet_center) with arc circle.
    let dir_to_fillet = try_normalize(fillet_center - arc.center)
        .ok_or_else(|| KernelError::Geometry("Fillet center coincides with arc center".into()))?;
    let tp_arc = arc.center + dir_to_fillet * arc.radius;

    // Insert new geometry.
    let center_pt = sketch.add_entity(SketchEntity::Point { position: fillet_center });
    let arc_start_pt = sketch.add_entity(SketchEntity::Point { position: tp_line });
    let arc_end_pt = sketch.add_entity(SketchEntity::Point { position: tp_arc });

    let fillet_arc_id = sketch.add_entity(SketchEntity::Arc {
        center: center_pt,
        start: arc_start_pt,
        end: arc_end_pt,
    });

    // Trim the line.
    trim_line_endpoint(sketch, line, vertex, arc_start_pt, tolerance)?;
    // Trim the arc.
    trim_arc_endpoint(sketch, arc, vertex, arc_end_pt, tolerance)?;

    Ok(SketchFilletResult {
        fillet_arc: fillet_arc_id,
        modified_entities: vec![line.id, arc.id],
        removed_entities: vec![],
    })
}

/// Fillet between two arcs meeting at `vertex`.
fn fillet_arc_arc(
    sketch: &mut Sketch,
    a: &ArcGeom,
    b: &ArcGeom,
    vertex: Pt2,
    radius: f64,
    tolerance: f64,
) -> KernelResult<SketchFilletResult> {
    // The fillet center is at distance (r_a +/- radius) from center_a
    // and at distance (r_b +/- radius) from center_b.
    // We try all 4 combinations and pick the best.

    let offsets_a = [a.radius + radius, (a.radius - radius).abs()];
    let offsets_b = [b.radius + radius, (b.radius - radius).abs()];

    let mut best_center: Option<Pt2> = None;
    let mut best_dist = f64::MAX;

    for &da in &offsets_a {
        for &db in &offsets_b {
            if da < 1e-15 || db < 1e-15 {
                continue;
            }
            if let Some(pts) = circle_circle_intersection(a.center, da, b.center, db) {
                for c in pts {
                    // Verify distances.
                    let dist_a = distance(c, a.center);
                    let dist_b = distance(c, b.center);
                    let err_a = (dist_a - da).abs();
                    let err_b = (dist_b - db).abs();
                    if err_a < tolerance * 10.0 && err_b < tolerance * 10.0 {
                        let d = distance(c, vertex);
                        if d < best_dist {
                            best_dist = d;
                            best_center = Some(c);
                        }
                    }
                }
            }
        }
    }

    let fillet_center = best_center.ok_or_else(|| KernelError::InvalidParameter {
        param: "radius".into(),
        value: format!("{} (no valid fillet center found for arc-arc corner)", radius),
    })?;

    // Tangent points on each arc.
    let dir_a = try_normalize(fillet_center - a.center)
        .ok_or_else(|| KernelError::Geometry("Fillet center coincides with arc A center".into()))?;
    let tp_a = a.center + dir_a * a.radius;

    let dir_b = try_normalize(fillet_center - b.center)
        .ok_or_else(|| KernelError::Geometry("Fillet center coincides with arc B center".into()))?;
    let tp_b = b.center + dir_b * b.radius;

    // Insert new geometry.
    let center_pt = sketch.add_entity(SketchEntity::Point { position: fillet_center });
    let arc_start_pt = sketch.add_entity(SketchEntity::Point { position: tp_a });
    let arc_end_pt = sketch.add_entity(SketchEntity::Point { position: tp_b });

    let fillet_arc_id = sketch.add_entity(SketchEntity::Arc {
        center: center_pt,
        start: arc_start_pt,
        end: arc_end_pt,
    });

    // Trim both arcs.
    trim_arc_endpoint(sketch, a, vertex, arc_start_pt, tolerance)?;
    trim_arc_endpoint(sketch, b, vertex, arc_end_pt, tolerance)?;

    Ok(SketchFilletResult {
        fillet_arc: fillet_arc_id,
        modified_entities: vec![a.id, b.id],
        removed_entities: vec![],
    })
}

// ─── Geometric helpers ───────────────────────────────────────────────────────

/// Direction along a line away from the vertex.
fn line_direction_from_vertex(line: &LineGeom, vertex: Pt2, tolerance: f64) -> Vec2 {
    if distance(line.start, vertex) < tolerance {
        line.end - line.start
    } else {
        line.start - line.end
    }
}

/// Available length of a line from vertex to the far endpoint.
fn line_available_length(line: &LineGeom, vertex: Pt2, tolerance: f64) -> f64 {
    if distance(line.start, vertex) < tolerance {
        distance(line.start, line.end)
    } else {
        distance(line.end, line.start)
    }
}

/// Intersect a line (point + direction) with a circle; returns 0-2 intersection points.
fn line_circle_intersection(
    line_pt: Pt2,
    line_dir: Vec2,
    circle_center: Pt2,
    circle_radius: f64,
) -> Option<Vec<Pt2>> {
    let oc = line_pt - circle_center;
    let a = line_dir.dot(&line_dir);
    let b = 2.0 * oc.dot(&line_dir);
    let c = oc.dot(&oc) - circle_radius * circle_radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < -1e-12 {
        return None;
    }

    let disc_sqrt = discriminant.max(0.0).sqrt();
    let t1 = (-b - disc_sqrt) / (2.0 * a);
    let t2 = (-b + disc_sqrt) / (2.0 * a);

    let mut points = Vec::new();
    points.push(line_pt + line_dir * t1);
    if (t2 - t1).abs() > 1e-12 {
        points.push(line_pt + line_dir * t2);
    }
    Some(points)
}

/// Intersect two circles; returns 0-2 intersection points.
fn circle_circle_intersection(
    c1: Pt2,
    r1: f64,
    c2: Pt2,
    r2: f64,
) -> Option<Vec<Pt2>> {
    let d = distance(c1, c2);
    if d < 1e-15 {
        return None; // Concentric
    }
    if d > r1 + r2 + 1e-9 || d < (r1 - r2).abs() - 1e-9 {
        return None; // No intersection
    }

    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h_sq = r1 * r1 - a * a;
    let h = if h_sq < 0.0 { 0.0 } else { h_sq.sqrt() };

    let dir = try_normalize(c2 - c1)?;
    let mid = c1 + dir * a;
    let perp = Vec2::new(-dir.y, dir.x);

    let mut points = Vec::new();
    points.push(mid + perp * h);
    if h > 1e-12 {
        points.push(mid - perp * h);
    }
    Some(points)
}

/// Trim a line by replacing the endpoint nearest to `vertex` with `new_point_id`.
fn trim_line_endpoint(
    sketch: &mut Sketch,
    line: &LineGeom,
    vertex: Pt2,
    new_point_id: SketchEntityId,
    tolerance: f64,
) -> KernelResult<()> {
    let entity = sketch.entities.get_mut(line.id)?;
    match entity {
        SketchEntity::Line { start, end } => {
            if distance(line.start, vertex) < tolerance {
                *start = new_point_id;
            } else {
                *end = new_point_id;
            }
            Ok(())
        }
        _ => Err(KernelError::Internal(
            "Expected Line entity during trim".into(),
        )),
    }
}

/// Trim an arc by replacing the endpoint nearest to `vertex` with `new_point_id`.
fn trim_arc_endpoint(
    sketch: &mut Sketch,
    arc: &ArcGeom,
    vertex: Pt2,
    new_point_id: SketchEntityId,
    tolerance: f64,
) -> KernelResult<()> {
    let entity = sketch.entities.get_mut(arc.id)?;
    match entity {
        SketchEntity::Arc { start, end, .. } => {
            if distance(arc.start, vertex) < tolerance {
                *start = new_point_id;
            } else {
                *end = new_point_id;
            }
            Ok(())
        }
        _ => Err(KernelError::Internal(
            "Expected Arc entity during trim".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    /// Helper to build a sketch with two perpendicular lines meeting at origin.
    fn make_right_angle_sketch() -> (Sketch, Pt2) {
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
        let _line_a = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: px,
        });
        let _line_b = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: py,
        });
        (sketch, Pt2::new(0.0, 0.0))
    }

    #[test]
    fn fillet_perpendicular_lines() {
        let (mut sketch, vertex) = make_right_angle_sketch();
        let result = sketch_fillet(&mut sketch, vertex, 2.0, 1e-6).unwrap();

        // Should have created a fillet arc.
        let arc_entity = sketch.entities.get(result.fillet_arc).unwrap();
        match arc_entity {
            SketchEntity::Arc { center, start, end } => {
                let c = get_point_position(&sketch, *center).unwrap();
                let s = get_point_position(&sketch, *start).unwrap();
                let e = get_point_position(&sketch, *end).unwrap();
                // Arc center should be at (2, 2) for a 90-degree corner.
                assert!((c.x - 2.0).abs() < 1e-6);
                assert!((c.y - 2.0).abs() < 1e-6);
                // Tangent points on axes.
                assert!((s.y).abs() < 1e-6); // On the X axis.
                assert!((e.x).abs() < 1e-6); // On the Y axis.
                // Both at radius distance from center.
                assert!((distance(c, s) - 2.0).abs() < 1e-6);
                assert!((distance(c, e) - 2.0).abs() < 1e-6);
            }
            _ => panic!("Expected Arc entity"),
        }

        assert_eq!(result.modified_entities.len(), 2);
    }

    #[test]
    fn fillet_angled_lines() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let origin = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        // Line along X axis.
        let px = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0, 0.0),
        });
        // Line at 60 degrees.
        let p60 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(10.0 * 0.5, 10.0 * (3.0_f64).sqrt() / 2.0),
        });
        let _l1 = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: px,
        });
        let _l2 = sketch.add_entity(SketchEntity::Line {
            start: origin,
            end: p60,
        });

        let vertex = Pt2::new(0.0, 0.0);
        let radius = 1.0;
        let result = sketch_fillet(&mut sketch, vertex, radius, 1e-6).unwrap();

        let arc = sketch.entities.get(result.fillet_arc).unwrap();
        match arc {
            SketchEntity::Arc { center, start, end } => {
                let c = get_point_position(&sketch, *center).unwrap();
                let s = get_point_position(&sketch, *start).unwrap();
                let e = get_point_position(&sketch, *end).unwrap();
                // Both tangent points should be at exactly radius from center.
                assert!((distance(c, s) - radius).abs() < 1e-6);
                assert!((distance(c, e) - radius).abs() < 1e-6);
            }
            _ => panic!("Expected Arc entity"),
        }
    }

    #[test]
    fn fillet_line_and_arc() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // An arc centered at (5, 0) with radius 5. The arc passes through origin.
        let arc_center = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 0.0),
        });
        let arc_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let arc_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(5.0, 5.0),
        });
        let _arc = sketch.add_entity(SketchEntity::Arc {
            center: arc_center,
            start: arc_start,
            end: arc_end,
        });

        // A line from origin going up.
        let line_start = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let line_end = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 10.0),
        });
        let _line = sketch.add_entity(SketchEntity::Line {
            start: line_start,
            end: line_end,
        });

        let vertex = Pt2::new(0.0, 0.0);
        let result = sketch_fillet(&mut sketch, vertex, 1.0, 1e-6).unwrap();

        let fillet = sketch.entities.get(result.fillet_arc).unwrap();
        match fillet {
            SketchEntity::Arc { center, start, end } => {
                let c = get_point_position(&sketch, *center).unwrap();
                let s = get_point_position(&sketch, *start).unwrap();
                let e = get_point_position(&sketch, *end).unwrap();
                // Tangent points should be at radius from center.
                assert!((distance(c, s) - 1.0).abs() < 1e-4);
                assert!((distance(c, e) - 1.0).abs() < 1e-4);
            }
            _ => panic!("Expected Arc entity"),
        }
    }

    #[test]
    fn fillet_radius_too_large() {
        let (mut sketch, vertex) = make_right_angle_sketch();
        // Lines are 10 units long; a radius of 20 should fail.
        let result = sketch_fillet(&mut sketch, vertex, 20.0, 1e-6);
        assert!(result.is_err());
    }

    #[test]
    fn fillet_creates_tangent_arc() {
        let (mut sketch, vertex) = make_right_angle_sketch();
        let radius = 3.0;
        let result = sketch_fillet(&mut sketch, vertex, radius, 1e-6).unwrap();

        let arc = sketch.entities.get(result.fillet_arc).unwrap();
        match arc {
            SketchEntity::Arc { center, start, end } => {
                let c = get_point_position(&sketch, *center).unwrap();
                let s = get_point_position(&sketch, *start).unwrap();
                let e = get_point_position(&sketch, *end).unwrap();

                // Arc radius matches requested radius.
                assert!((distance(c, s) - radius).abs() < 1e-9);
                assert!((distance(c, e) - radius).abs() < 1e-9);

                // Tangent point on X axis: perpendicular from center to X axis.
                // s should be on the X axis, so s.y == 0.
                assert!(s.y.abs() < 1e-9, "Tangent point should lie on X axis");
                // e should be on the Y axis, so e.x == 0.
                assert!(e.x.abs() < 1e-9, "Tangent point should lie on Y axis");

                // Center at (radius, radius) for a right-angle corner at origin.
                assert!((c.x - radius).abs() < 1e-9);
                assert!((c.y - radius).abs() < 1e-9);
            }
            _ => panic!("Expected Arc entity"),
        }
    }

    #[test]
    fn fillet_zero_radius_errors() {
        let (mut sketch, vertex) = make_right_angle_sketch();
        let result = sketch_fillet(&mut sketch, vertex, 0.0, 1e-6);
        assert!(result.is_err());
    }

    #[test]
    fn fillet_negative_radius_errors() {
        let (mut sketch, vertex) = make_right_angle_sketch();
        let result = sketch_fillet(&mut sketch, vertex, -1.0, 1e-6);
        assert!(result.is_err());
    }

    #[test]
    fn fillet_no_entities_at_vertex() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        // No entities near this point.
        let result = sketch_fillet(&mut sketch, Pt2::new(5.0, 5.0), 1.0, 1e-6);
        assert!(result.is_err());
    }
}
