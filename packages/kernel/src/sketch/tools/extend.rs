//! Extend sketch entities (lines, arcs) to meet boundary entities.

use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt2;
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Which end of the entity to extend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtendEnd {
    Start,
    End,
}

/// Result of an extend operation.
#[derive(Debug, Clone)]
pub struct ExtendResult {
    pub entity_id: SketchEntityId,
    pub new_endpoint: Pt2,
}

/// Extend a sketch entity (line or arc) to meet the nearest boundary entity.
///
/// The entity is extended from the specified end along its natural direction
/// until it intersects another entity in the sketch. If no boundary is found,
/// an error is returned.
pub fn extend_entity(
    sketch: &mut Sketch,
    entity_id: SketchEntityId,
    end: ExtendEnd,
    tolerance: f64,
) -> KernelResult<ExtendResult> {
    // Resolve the entity and compute extension candidates.
    let entity = sketch.entities.get(entity_id)?.clone();

    let (extension_points, _boundary_ids): (Vec<Vec<Pt2>>, Vec<SketchEntityId>) = match &entity {
        SketchEntity::Line { start, end: end_id } => {
            let start_pos = get_point_position(&sketch, *start)?;
            let end_pos = get_point_position(&sketch, *end_id)?;
            find_line_extension_candidates(sketch, entity_id, start_pos, end_pos, end, tolerance)?
        }
        SketchEntity::Arc {
            center,
            start,
            end: end_id,
        } => {
            let center_pos = get_point_position(&sketch, *center)?;
            let start_pos = get_point_position(&sketch, *start)?;
            let end_pos = get_point_position(&sketch, *end_id)?;
            find_arc_extension_candidates(
                sketch, entity_id, center_pos, start_pos, end_pos, end, tolerance,
            )?
        }
        _ => {
            return Err(KernelError::Operation {
                op: "extend".into(),
                detail: "Only lines and arcs can be extended".into(),
            });
        }
    };

    // Pick the nearest intersection point across all boundary candidates.
    let extending_from = match &entity {
        SketchEntity::Line { start, end: end_id } => match end {
            ExtendEnd::Start => get_point_position(&sketch, *start)?,
            ExtendEnd::End => get_point_position(&sketch, *end_id)?,
        },
        SketchEntity::Arc {
            start, end: end_id, ..
        } => match end {
            ExtendEnd::Start => get_point_position(&sketch, *start)?,
            ExtendEnd::End => get_point_position(&sketch, *end_id)?,
        },
        _ => unreachable!(),
    };

    let mut best: Option<(Pt2, f64)> = None;
    for pts in &extension_points {
        for &pt in pts {
            let dist = (pt - extending_from).norm();
            if dist > tolerance {
                if best.is_none() || dist < best.unwrap().1 {
                    best = Some((pt, dist));
                }
            }
        }
    }

    let new_endpoint = best
        .map(|(pt, _)| pt)
        .ok_or_else(|| KernelError::Operation {
            op: "extend".into(),
            detail: "No boundary entity found to extend to".into(),
        })?;

    // Update the endpoint of the entity.
    apply_extension(sketch, entity_id, end, new_endpoint)?;

    Ok(ExtendResult {
        entity_id,
        new_endpoint,
    })
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_point_position(sketch: &Sketch, id: SketchEntityId) -> KernelResult<Pt2> {
    match sketch.entities.get(id)? {
        SketchEntity::Point { position } => Ok(*position),
        _ => Err(KernelError::Operation {
            op: "extend".into(),
            detail: "Expected a point entity".into(),
        }),
    }
}

/// Update the position of the point at the specified end of the entity.
fn apply_extension(
    sketch: &mut Sketch,
    entity_id: SketchEntityId,
    end: ExtendEnd,
    new_pos: Pt2,
) -> KernelResult<()> {
    let entity = sketch.entities.get(entity_id)?.clone();
    let point_id = match (&entity, end) {
        (SketchEntity::Line { start, .. }, ExtendEnd::Start) => *start,
        (SketchEntity::Line { end: e, .. }, ExtendEnd::End) => *e,
        (SketchEntity::Arc { start, .. }, ExtendEnd::Start) => *start,
        (SketchEntity::Arc { end: e, .. }, ExtendEnd::End) => *e,
        _ => {
            return Err(KernelError::Operation {
                op: "extend".into(),
                detail: "Unsupported entity type for extension".into(),
            });
        }
    };

    let point = sketch.entities.get_mut(point_id)?;
    match point {
        SketchEntity::Point { position } => {
            *position = new_pos;
            Ok(())
        }
        _ => Err(KernelError::Operation {
            op: "extend".into(),
            detail: "Expected a point entity".into(),
        }),
    }
}

// ---------------------------------------------------------------------------
// Line extension
// ---------------------------------------------------------------------------

/// For a line defined by `start_pos` -> `end_pos`, find intersection points
/// with all other entities when extending from the specified end.
fn find_line_extension_candidates(
    sketch: &Sketch,
    entity_id: SketchEntityId,
    start_pos: Pt2,
    end_pos: Pt2,
    end: ExtendEnd,
    tolerance: f64,
) -> KernelResult<(Vec<Vec<Pt2>>, Vec<SketchEntityId>)> {
    let dir = match end {
        ExtendEnd::End => end_pos - start_pos,
        ExtendEnd::Start => start_pos - end_pos,
    };
    if dir.norm() < tolerance {
        return Err(KernelError::Operation {
            op: "extend".into(),
            detail: "Line has zero length".into(),
        });
    }
    let dir = dir.normalize();
    let origin = match end {
        ExtendEnd::End => end_pos,
        ExtendEnd::Start => start_pos,
    };

    let mut all_points = Vec::new();
    let mut boundary_ids = Vec::new();

    for (bid, bentity) in sketch.entities.iter() {
        if bid == entity_id {
            continue;
        }
        let pts = line_intersect_entity(sketch, origin, dir, bid, bentity, tolerance)?;
        if !pts.is_empty() {
            // Filter to only points that are in the extension direction (positive t).
            let forward: Vec<Pt2> = pts
                .into_iter()
                .filter(|p| {
                    let t = (*p - origin).dot(&dir);
                    t > tolerance
                })
                .collect();
            if !forward.is_empty() {
                all_points.push(forward);
                boundary_ids.push(bid);
            }
        }
    }

    Ok((all_points, boundary_ids))
}

/// Compute intersections of a ray (origin + t*dir, t >= 0) with a sketch entity.
fn line_intersect_entity(
    sketch: &Sketch,
    origin: Pt2,
    dir: nalgebra::Vector2<f64>,
    _boundary_id: SketchEntityId,
    boundary: &SketchEntity,
    tolerance: f64,
) -> KernelResult<Vec<Pt2>> {
    match boundary {
        SketchEntity::Line { start, end } => {
            let p = get_point_position(sketch, *start)?;
            let q = get_point_position(sketch, *end)?;
            Ok(ray_segment_intersection(origin, dir, p, q, tolerance))
        }
        SketchEntity::Circle { center, radius } => {
            let c = get_point_position(sketch, *center)?;
            Ok(ray_circle_intersection(origin, dir, c, *radius, tolerance))
        }
        SketchEntity::Arc {
            center,
            start,
            end: end_id,
        } => {
            let c = get_point_position(sketch, *center)?;
            let s = get_point_position(sketch, *start)?;
            let e = get_point_position(sketch, *end_id)?;
            let radius = (s - c).norm();
            // Get full circle intersections then filter to arc span.
            let circle_pts = ray_circle_intersection(origin, dir, c, radius, tolerance);
            Ok(filter_points_on_arc(c, s, e, &circle_pts))
        }
        _ => Ok(vec![]),
    }
}

// ---------------------------------------------------------------------------
// Arc extension
// ---------------------------------------------------------------------------

/// For an arc, extend along the circle and find intersections with boundaries.
fn find_arc_extension_candidates(
    sketch: &Sketch,
    entity_id: SketchEntityId,
    center: Pt2,
    start_pos: Pt2,
    end_pos: Pt2,
    end: ExtendEnd,
    tolerance: f64,
) -> KernelResult<(Vec<Vec<Pt2>>, Vec<SketchEntityId>)> {
    let radius = (start_pos - center).norm();
    if radius < tolerance {
        return Err(KernelError::Operation {
            op: "extend".into(),
            detail: "Arc has zero radius".into(),
        });
    }

    // The extension point on the arc and its tangent direction.
    let ext_point = match end {
        ExtendEnd::End => end_pos,
        ExtendEnd::Start => start_pos,
    };

    // Arc angle for the extension endpoint.
    let ext_angle = (ext_point.y - center.y).atan2(ext_point.x - center.x);

    // Arc sweep direction: start_angle -> end_angle, CCW.
    let start_angle = (start_pos.y - center.y).atan2(start_pos.x - center.x);
    let end_angle = (end_pos.y - center.y).atan2(end_pos.x - center.x);

    let mut all_points = Vec::new();
    let mut boundary_ids = Vec::new();

    for (bid, bentity) in sketch.entities.iter() {
        if bid == entity_id {
            continue;
        }
        let pts =
            arc_intersect_entity(sketch, center, radius, bid, bentity, tolerance)?;
        if !pts.is_empty() {
            // Filter: only points on the extension portion of the circle
            // (beyond the current arc span, in the extension direction).
            let filtered: Vec<Pt2> = pts
                .into_iter()
                .filter(|p| {
                    let angle = (p.y - center.y).atan2(p.x - center.x);
                    is_in_extension_arc(start_angle, end_angle, ext_angle, angle, end)
                })
                .collect();
            if !filtered.is_empty() {
                // Sort by angular distance from the extending end.
                let mut sorted = filtered;
                sorted.sort_by(|a, b| {
                    let a_angle = (a.y - center.y).atan2(a.x - center.x);
                    let b_angle = (b.y - center.y).atan2(b.x - center.x);
                    let a_dist = angular_distance(ext_angle, a_angle, end);
                    let b_dist = angular_distance(ext_angle, b_angle, end);
                    a_dist.partial_cmp(&b_dist).unwrap()
                });
                all_points.push(sorted);
                boundary_ids.push(bid);
            }
        }
    }

    Ok((all_points, boundary_ids))
}

/// Find intersections of a full circle with a sketch entity.
fn arc_intersect_entity(
    sketch: &Sketch,
    center: Pt2,
    radius: f64,
    _boundary_id: SketchEntityId,
    boundary: &SketchEntity,
    tolerance: f64,
) -> KernelResult<Vec<Pt2>> {
    match boundary {
        SketchEntity::Line { start, end } => {
            let p = get_point_position(sketch, *start)?;
            let q = get_point_position(sketch, *end)?;
            Ok(circle_line_segment_intersection(center, radius, p, q, tolerance))
        }
        SketchEntity::Circle {
            center: c2,
            radius: r2,
        } => {
            let c2_pos = get_point_position(sketch, *c2)?;
            Ok(circle_circle_intersection(center, radius, c2_pos, *r2, tolerance))
        }
        SketchEntity::Arc {
            center: c2,
            start,
            end: end_id,
        } => {
            let c2_pos = get_point_position(sketch, *c2)?;
            let s = get_point_position(sketch, *start)?;
            let e = get_point_position(sketch, *end_id)?;
            let r2 = (s - c2_pos).norm();
            let pts = circle_circle_intersection(center, radius, c2_pos, r2, tolerance);
            Ok(filter_points_on_arc(c2_pos, s, e, &pts))
        }
        _ => Ok(vec![]),
    }
}

/// Check if `angle` is in the extension region of the arc (beyond the current span).
fn is_in_extension_arc(
    start_angle: f64,
    end_angle: f64,
    ext_angle: f64,
    angle: f64,
    end: ExtendEnd,
) -> bool {
    let _ = (start_angle, end_angle); // arc span reference
    // When extending from End, we go CCW beyond end_angle.
    // When extending from Start, we go CW beyond start_angle.
    let diff = match end {
        ExtendEnd::End => normalize_angle(angle - ext_angle),
        ExtendEnd::Start => normalize_angle(ext_angle - angle),
    };
    // The point should be beyond the arc endpoint but within a full revolution.
    diff > 1e-9 && diff < std::f64::consts::TAU - 1e-9
}

/// Compute angular distance from `from` to `to` in the extension direction.
fn angular_distance(from: f64, to: f64, end: ExtendEnd) -> f64 {
    match end {
        ExtendEnd::End => normalize_angle(to - from),
        ExtendEnd::Start => normalize_angle(from - to),
    }
}

/// Normalize angle to [0, 2*PI).
fn normalize_angle(a: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    ((a % tau) + tau) % tau
}

// ---------------------------------------------------------------------------
// Geometric intersection routines
// ---------------------------------------------------------------------------

/// Intersection of a ray (origin + t*dir) with a line segment (p -> q).
/// Returns the intersection point if t >= 0 and the point lies on the segment.
fn ray_segment_intersection(
    origin: Pt2,
    dir: nalgebra::Vector2<f64>,
    p: Pt2,
    q: Pt2,
    tolerance: f64,
) -> Vec<Pt2> {
    let seg = q - p;
    let denom = dir.x * seg.y - dir.y * seg.x;
    if denom.abs() < tolerance {
        return vec![]; // parallel
    }
    let dp = p - origin;
    let t = (dp.x * seg.y - dp.y * seg.x) / denom;
    let u = (dp.x * dir.y - dp.y * dir.x) / denom;

    if t > -tolerance && u > -tolerance && u < 1.0 + tolerance {
        vec![origin + dir * t]
    } else {
        vec![]
    }
}

/// Intersection of a ray with a circle.
fn ray_circle_intersection(
    origin: Pt2,
    dir: nalgebra::Vector2<f64>,
    center: Pt2,
    radius: f64,
    tolerance: f64,
) -> Vec<Pt2> {
    let oc = origin - center;
    let a = dir.dot(&dir);
    let b = 2.0 * oc.dot(&dir);
    let c = oc.dot(&oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < -tolerance {
        return vec![];
    }
    let disc = disc.max(0.0).sqrt();
    let mut result = Vec::new();
    for &t in &[(-b - disc) / (2.0 * a), (-b + disc) / (2.0 * a)] {
        if t > -tolerance {
            result.push(origin + dir * t);
        }
    }
    result
}

/// Intersection of a circle with a line segment.
fn circle_line_segment_intersection(
    center: Pt2,
    radius: f64,
    p: Pt2,
    q: Pt2,
    tolerance: f64,
) -> Vec<Pt2> {
    let seg = q - p;
    let len = seg.norm();
    if len < tolerance {
        return vec![];
    }
    let dir = seg / len;
    // Use ray-circle and filter to segment parameter [0, len].
    let oc = p - center;
    let a = 1.0; // dir is unit
    let b = 2.0 * oc.dot(&dir);
    let c = oc.dot(&oc) - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < -tolerance {
        return vec![];
    }
    let disc = disc.max(0.0).sqrt();
    let mut result = Vec::new();
    for &t in &[(-b - disc) / 2.0, (-b + disc) / 2.0] {
        if t > -tolerance && t < len + tolerance {
            result.push(p + dir * t);
        }
    }
    result
}

/// Intersection of two circles.
fn circle_circle_intersection(
    c1: Pt2,
    r1: f64,
    c2: Pt2,
    r2: f64,
    tolerance: f64,
) -> Vec<Pt2> {
    let d_vec = c2 - c1;
    let d = d_vec.norm();
    if d < tolerance {
        return vec![]; // concentric
    }
    if d > r1 + r2 + tolerance || d < (r1 - r2).abs() - tolerance {
        return vec![]; // no intersection
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h_sq = r1 * r1 - a * a;
    if h_sq < -tolerance {
        return vec![];
    }
    let h = h_sq.max(0.0).sqrt();
    let mid = c1 + d_vec * (a / d);
    let perp = nalgebra::Vector2::new(-d_vec.y, d_vec.x) / d;

    if h < tolerance {
        vec![mid]
    } else {
        vec![mid + perp * h, mid - perp * h]
    }
}

/// Filter intersection points to those lying on an arc from `start` to `end` (CCW).
fn filter_points_on_arc(center: Pt2, start: Pt2, end: Pt2, pts: &[Pt2]) -> Vec<Pt2> {
    let start_angle = (start.y - center.y).atan2(start.x - center.x);
    let end_angle = (end.y - center.y).atan2(end.x - center.x);
    let sweep = normalize_angle(end_angle - start_angle);

    pts.iter()
        .copied()
        .filter(|p| {
            let angle = (p.y - center.y).atan2(p.x - center.x);
            let from_start = normalize_angle(angle - start_angle);
            from_start <= sweep + 1e-9
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;

    fn make_sketch() -> Sketch {
        Sketch::new(Plane::xy(0.0))
    }

    fn add_point(sketch: &mut Sketch, x: f64, y: f64) -> SketchEntityId {
        sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(x, y),
        })
    }

    fn add_line(sketch: &mut Sketch, x1: f64, y1: f64, x2: f64, y2: f64) -> SketchEntityId {
        let p1 = add_point(sketch, x1, y1);
        let p2 = add_point(sketch, x2, y2);
        sketch.add_entity(SketchEntity::Line { start: p1, end: p2 })
    }

    fn add_circle(sketch: &mut Sketch, cx: f64, cy: f64, r: f64) -> SketchEntityId {
        let c = add_point(sketch, cx, cy);
        sketch.add_entity(SketchEntity::Circle {
            center: c,
            radius: r,
        })
    }

    fn add_arc(
        sketch: &mut Sketch,
        cx: f64,
        cy: f64,
        sx: f64,
        sy: f64,
        ex: f64,
        ey: f64,
    ) -> SketchEntityId {
        let c = add_point(sketch, cx, cy);
        let s = add_point(sketch, sx, sy);
        let e = add_point(sketch, ex, ey);
        sketch.add_entity(SketchEntity::Arc {
            center: c,
            start: s,
            end: e,
        })
    }

    #[test]
    fn extend_line_to_line_end() {
        // Line A: (0,0) -> (3,0), to be extended from End
        // Line B: vertical at x=5: (5,-2) -> (5,2)
        // Expected: extend A to (5,0)
        let mut sketch = make_sketch();
        let line_a = add_line(&mut sketch, 0.0, 0.0, 3.0, 0.0);
        let _line_b = add_line(&mut sketch, 5.0, -2.0, 5.0, 2.0);

        let result = extend_entity(&mut sketch, line_a, ExtendEnd::End, 1e-9).unwrap();
        assert!((result.new_endpoint.x - 5.0).abs() < 1e-6);
        assert!((result.new_endpoint.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn extend_line_to_line_start() {
        // Line A: (3,0) -> (6,0), to be extended from Start
        // Line B: vertical at x=1: (1,-2) -> (1,2)
        // Expected: extend A's start to (1,0)
        let mut sketch = make_sketch();
        let line_a = add_line(&mut sketch, 3.0, 0.0, 6.0, 0.0);
        let _line_b = add_line(&mut sketch, 1.0, -2.0, 1.0, 2.0);

        let result = extend_entity(&mut sketch, line_a, ExtendEnd::Start, 1e-9).unwrap();
        assert!((result.new_endpoint.x - 1.0).abs() < 1e-6);
        assert!((result.new_endpoint.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn extend_line_to_circle() {
        // Line: (0,0) -> (2,0), extend End
        // Circle: center (5,0), radius 1  =>  left-most point is (4,0)
        let mut sketch = make_sketch();
        let line = add_line(&mut sketch, 0.0, 0.0, 2.0, 0.0);
        let _circle = add_circle(&mut sketch, 5.0, 0.0, 1.0);

        let result = extend_entity(&mut sketch, line, ExtendEnd::End, 1e-9).unwrap();
        // Should hit the near side of the circle at x=4
        assert!((result.new_endpoint.x - 4.0).abs() < 1e-6);
        assert!((result.new_endpoint.y - 0.0).abs() < 1e-6);
    }

    #[test]
    fn extend_arc_to_line() {
        // Arc: center (0,0), from (1,0) to (0,1) (quarter circle CCW in Q1).
        // Boundary: line at y = -0.5, from (-2, -0.5) to (2, -0.5).
        // Extending from Start (going CW beyond (1,0)), the arc should hit
        // y=-0.5 at angle = -pi/6... Let's compute:
        // angle for y=-0.5 on unit circle: sin(a) = -0.5 => a = -pi/6 => x = cos(-pi/6) = sqrt(3)/2
        let mut sketch = make_sketch();
        let arc = add_arc(&mut sketch, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0);
        let _boundary = add_line(&mut sketch, -2.0, -0.5, 2.0, -0.5);

        let result = extend_entity(&mut sketch, arc, ExtendEnd::Start, 1e-9).unwrap();
        let expected_x = (std::f64::consts::FRAC_PI_6).cos(); // sqrt(3)/2
        let expected_y = -0.5;
        assert!(
            (result.new_endpoint.x - expected_x).abs() < 1e-6,
            "x: got {}, expected {}",
            result.new_endpoint.x,
            expected_x
        );
        assert!(
            (result.new_endpoint.y - expected_y).abs() < 1e-6,
            "y: got {}, expected {}",
            result.new_endpoint.y,
            expected_y
        );
    }

    #[test]
    fn extend_no_boundary_returns_error() {
        // Line with no other entities => should error
        let mut sketch = make_sketch();
        let line = add_line(&mut sketch, 0.0, 0.0, 1.0, 0.0);

        let result = extend_entity(&mut sketch, line, ExtendEnd::End, 1e-9);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("No boundary"),
            "Expected 'No boundary' error, got: {}",
            err
        );
    }
}
