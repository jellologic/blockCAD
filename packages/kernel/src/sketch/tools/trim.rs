//! Trim entities sketch tool.
//!
//! Trims a sketch entity at its intersection points with other entities,
//! removing the segment nearest to the click point.

use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt2;
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::sketch::Sketch;

/// Result of a trim operation describing what changed in the sketch.
#[derive(Debug, Clone)]
pub struct TrimResult {
    /// Entities that were removed from the sketch.
    pub removed_entities: Vec<SketchEntityId>,
    /// New entities that were added (e.g. the remaining segments after trimming).
    pub added_entities: Vec<SketchEntityId>,
    /// Entities that were modified in-place.
    pub modified_entities: Vec<SketchEntityId>,
}

/// Trim the nearest sketch entity at the click point.
///
/// Finds the closest entity to `click_point` (within `tolerance`), computes
/// intersection points with all other entities in the sketch, and removes the
/// segment of the entity that is closest to the click point.
///
/// - If the entity has no intersections with other entities, it is removed entirely.
/// - If a circle is trimmed, it is converted to an arc spanning the remaining portion.
/// - Lines and arcs are split at the two bounding intersection points and the
///   middle segment (closest to the click) is removed.
pub fn trim_entity(
    sketch: &mut Sketch,
    click_point: Pt2,
    tolerance: f64,
) -> KernelResult<TrimResult> {
    let (target_id, target_entity) = find_nearest_entity(sketch, click_point, tolerance)?;

    // Collect all other entities for intersection testing.
    let others: Vec<(SketchEntityId, SketchEntity)> = sketch
        .entities
        .iter()
        .filter(|(id, _)| *id != target_id)
        .map(|(id, e)| (id, e.clone()))
        .collect();

    // Resolve the target entity into geometric form so we can compute intersections.
    let target_geom = resolve_entity_geometry(sketch, &target_entity)?;

    // Gather all intersection points.
    let mut intersection_params: Vec<f64> = Vec::new();
    for (_other_id, other_entity) in &others {
        let other_geom = match resolve_entity_geometry(sketch, other_entity) {
            Ok(g) => g,
            Err(_) => continue, // skip non-geometric entities (Points, Splines, etc.)
        };
        let pts = find_intersections(&target_geom, &other_geom);
        for pt in pts {
            if let Some(t) = param_on_entity(&target_geom, &pt) {
                intersection_params.push(t);
            }
        }
    }

    // Deduplicate and sort parameters.
    intersection_params.sort_by(|a, b| a.partial_cmp(b).unwrap());
    intersection_params.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    // If no intersections, remove the entire entity (and its associated points).
    if intersection_params.is_empty() {
        remove_entity_and_points(sketch, target_id, &target_entity)?;
        return Ok(TrimResult {
            removed_entities: vec![target_id],
            added_entities: vec![],
            modified_entities: vec![],
        });
    }

    // Find the parameter of the click point on the entity.
    let click_t = param_on_entity(&target_geom, &click_point).unwrap_or(0.5);

    // Determine which segment the click falls in.
    match &target_geom {
        EntityGeometry::Line { start, end } => {
            trim_line(sketch, target_id, start, end, &intersection_params, click_t)
        }
        EntityGeometry::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => trim_arc(
            sketch,
            target_id,
            center,
            *radius,
            *start_angle,
            *end_angle,
            &intersection_params,
            click_t,
        ),
        EntityGeometry::Circle { center, radius } => trim_circle(
            sketch,
            target_id,
            center,
            *radius,
            &intersection_params,
            click_t,
        ),
    }
}

// ---------------------------------------------------------------------------
// Internal geometry representation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum EntityGeometry {
    Line {
        start: Pt2,
        end: Pt2,
    },
    Arc {
        center: Pt2,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },
    Circle {
        center: Pt2,
        radius: f64,
    },
}

/// Resolve a `SketchEntity` into its concrete geometry by looking up referenced
/// point positions in the sketch entity store.
fn resolve_entity_geometry(
    sketch: &Sketch,
    entity: &SketchEntity,
) -> KernelResult<EntityGeometry> {
    match entity {
        SketchEntity::Line { start, end } => {
            let sp = get_point_position(sketch, *start)?;
            let ep = get_point_position(sketch, *end)?;
            Ok(EntityGeometry::Line { start: sp, end: ep })
        }
        SketchEntity::Arc { center, start, end } => {
            let cp = get_point_position(sketch, *center)?;
            let sp = get_point_position(sketch, *start)?;
            let ep = get_point_position(sketch, *end)?;
            let radius = ((sp.x - cp.x).powi(2) + (sp.y - cp.y).powi(2)).sqrt();
            let start_angle = (sp.y - cp.y).atan2(sp.x - cp.x);
            let end_angle = (ep.y - cp.y).atan2(ep.x - cp.x);
            Ok(EntityGeometry::Arc {
                center: cp,
                radius,
                start_angle,
                end_angle,
            })
        }
        SketchEntity::Circle { center, radius } => {
            let cp = get_point_position(sketch, *center)?;
            Ok(EntityGeometry::Circle {
                center: cp,
                radius: *radius,
            })
        }
        _ => Err(KernelError::Operation {
            op: "trim".into(),
            detail: "Only Line, Arc, and Circle entities can be trimmed".into(),
        }),
    }
}

fn get_point_position(sketch: &Sketch, id: SketchEntityId) -> KernelResult<Pt2> {
    match sketch.entities.get(id)? {
        SketchEntity::Point { position } => Ok(*position),
        _ => Err(KernelError::Operation {
            op: "trim".into(),
            detail: format!("Entity {:?} is not a point", id),
        }),
    }
}

// ---------------------------------------------------------------------------
// Nearest entity search
// ---------------------------------------------------------------------------

/// Find the entity nearest to `point` within `tolerance`.
fn find_nearest_entity(
    sketch: &Sketch,
    point: Pt2,
    tolerance: f64,
) -> KernelResult<(SketchEntityId, SketchEntity)> {
    let mut best: Option<(SketchEntityId, SketchEntity, f64)> = None;

    for (id, entity) in sketch.entities.iter() {
        let geom = match resolve_entity_geometry(sketch, entity) {
            Ok(g) => g,
            Err(_) => continue, // skip non-trimmable entities
        };
        let dist = distance_to_entity(&geom, &point);
        if dist <= tolerance {
            if best.as_ref().map_or(true, |(_, _, d)| dist < *d) {
                best = Some((id, entity.clone(), dist));
            }
        }
    }

    best.map(|(id, entity, _)| (id, entity))
        .ok_or_else(|| KernelError::NotFound("No trimmable entity found near click point".into()))
}

/// Minimum distance from a point to an entity.
fn distance_to_entity(geom: &EntityGeometry, point: &Pt2) -> f64 {
    match geom {
        EntityGeometry::Line { start, end } => point_to_segment_distance(point, start, end),
        EntityGeometry::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => point_to_arc_distance(point, center, *radius, *start_angle, *end_angle),
        EntityGeometry::Circle { center, radius } => {
            let d = ((point.x - center.x).powi(2) + (point.y - center.y).powi(2)).sqrt();
            (d - radius).abs()
        }
    }
}

fn point_to_segment_distance(p: &Pt2, a: &Pt2, b: &Pt2) -> f64 {
    let ab_x = b.x - a.x;
    let ab_y = b.y - a.y;
    let ap_x = p.x - a.x;
    let ap_y = p.y - a.y;
    let ab_len_sq = ab_x * ab_x + ab_y * ab_y;
    if ab_len_sq < 1e-18 {
        return (ap_x * ap_x + ap_y * ap_y).sqrt();
    }
    let t = ((ap_x * ab_x + ap_y * ab_y) / ab_len_sq).clamp(0.0, 1.0);
    let proj_x = a.x + t * ab_x;
    let proj_y = a.y + t * ab_y;
    ((p.x - proj_x).powi(2) + (p.y - proj_y).powi(2)).sqrt()
}

fn point_to_arc_distance(
    p: &Pt2,
    center: &Pt2,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> f64 {
    let angle = (p.y - center.y).atan2(p.x - center.x);
    if angle_in_arc(angle, start_angle, end_angle) {
        let d = ((p.x - center.x).powi(2) + (p.y - center.y).powi(2)).sqrt();
        (d - radius).abs()
    } else {
        // Distance to nearest endpoint
        let s = Pt2::new(
            center.x + radius * start_angle.cos(),
            center.y + radius * start_angle.sin(),
        );
        let e = Pt2::new(
            center.x + radius * end_angle.cos(),
            center.y + radius * end_angle.sin(),
        );
        let ds = ((p.x - s.x).powi(2) + (p.y - s.y).powi(2)).sqrt();
        let de = ((p.x - e.x).powi(2) + (p.y - e.y).powi(2)).sqrt();
        ds.min(de)
    }
}

/// Check whether `angle` lies within the arc from `start_angle` to `end_angle`
/// (counter-clockwise).
fn angle_in_arc(angle: f64, start: f64, end: f64) -> bool {
    let a = normalize_angle(angle - start);
    let span = normalize_angle(end - start);
    // If span is ~0 treat as full circle
    if span.abs() < 1e-12 {
        return true;
    }
    a <= span + 1e-9
}

fn normalize_angle(a: f64) -> f64 {
    let mut r = a % (2.0 * std::f64::consts::PI);
    if r < 0.0 {
        r += 2.0 * std::f64::consts::PI;
    }
    r
}

// ---------------------------------------------------------------------------
// Parameterization helpers
// ---------------------------------------------------------------------------

/// Compute the parameter `t` (in [0,1] for lines/arcs, [0,2pi) for circles)
/// of a point on an entity.
fn param_on_entity(geom: &EntityGeometry, point: &Pt2) -> Option<f64> {
    match geom {
        EntityGeometry::Line { start, end } => {
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let len_sq = dx * dx + dy * dy;
            if len_sq < 1e-18 {
                return Some(0.0);
            }
            let t = ((point.x - start.x) * dx + (point.y - start.y) * dy) / len_sq;
            Some(t.clamp(0.0, 1.0))
        }
        EntityGeometry::Arc {
            center,
            start_angle,
            end_angle,
            ..
        } => {
            let angle = (point.y - center.y).atan2(point.x - center.x);
            let span = normalize_angle(*end_angle - *start_angle);
            if span.abs() < 1e-12 {
                return Some(0.0);
            }
            let t = normalize_angle(angle - *start_angle) / span;
            Some(t.clamp(0.0, 1.0))
        }
        EntityGeometry::Circle { center, .. } => {
            let angle = (point.y - center.y).atan2(point.x - center.x);
            // Normalize to [0, 2pi)
            Some(normalize_angle(angle))
        }
    }
}

/// Evaluate a point on an entity at parameter `t`.
#[allow(dead_code)]
fn point_at_param(geom: &EntityGeometry, t: f64) -> Pt2 {
    match geom {
        EntityGeometry::Line { start, end } => Pt2::new(
            start.x + t * (end.x - start.x),
            start.y + t * (end.y - start.y),
        ),
        EntityGeometry::Arc {
            center,
            radius,
            start_angle,
            end_angle,
            ..
        } => {
            let span = normalize_angle(*end_angle - *start_angle);
            let angle = *start_angle + t * span;
            Pt2::new(center.x + radius * angle.cos(), center.y + radius * angle.sin())
        }
        EntityGeometry::Circle { center, radius } => {
            // t is angle in radians [0, 2pi)
            Pt2::new(center.x + radius * t.cos(), center.y + radius * t.sin())
        }
    }
}

// ---------------------------------------------------------------------------
// Intersection routines
// ---------------------------------------------------------------------------

/// Find intersection points between two geometric entities.
fn find_intersections(a: &EntityGeometry, b: &EntityGeometry) -> Vec<Pt2> {
    match (a, b) {
        (EntityGeometry::Line { start: a1, end: a2 }, EntityGeometry::Line { start: b1, end: b2 }) => {
            line_line_intersect(a1, a2, b1, b2)
        }
        (
            EntityGeometry::Line { start, end },
            EntityGeometry::Circle { center, radius },
        )
        | (
            EntityGeometry::Circle { center, radius },
            EntityGeometry::Line { start, end },
        ) => line_circle_intersect(start, end, center, *radius),
        (
            EntityGeometry::Line { start, end },
            EntityGeometry::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            },
        )
        | (
            EntityGeometry::Arc {
                center,
                radius,
                start_angle,
                end_angle,
            },
            EntityGeometry::Line { start, end },
        ) => line_arc_intersect(start, end, center, *radius, *start_angle, *end_angle),
        (
            EntityGeometry::Circle {
                center: c1,
                radius: r1,
            },
            EntityGeometry::Circle {
                center: c2,
                radius: r2,
            },
        ) => circle_circle_intersect(c1, *r1, c2, *r2),
        (
            EntityGeometry::Circle {
                center: cc,
                radius: rc,
            },
            EntityGeometry::Arc {
                center: ca,
                radius: ra,
                start_angle,
                end_angle,
            },
        )
        | (
            EntityGeometry::Arc {
                center: ca,
                radius: ra,
                start_angle,
                end_angle,
            },
            EntityGeometry::Circle {
                center: cc,
                radius: rc,
            },
        ) => {
            let pts = circle_circle_intersect(cc, *rc, ca, *ra);
            pts.into_iter()
                .filter(|p| {
                    let angle = (p.y - ca.y).atan2(p.x - ca.x);
                    angle_in_arc(angle, *start_angle, *end_angle)
                })
                .collect()
        }
        (
            EntityGeometry::Arc {
                center: c1,
                radius: r1,
                start_angle: sa1,
                end_angle: ea1,
            },
            EntityGeometry::Arc {
                center: c2,
                radius: r2,
                start_angle: sa2,
                end_angle: ea2,
            },
        ) => {
            let pts = circle_circle_intersect(c1, *r1, c2, *r2);
            pts.into_iter()
                .filter(|p| {
                    let a1 = (p.y - c1.y).atan2(p.x - c1.x);
                    let a2 = (p.y - c2.y).atan2(p.x - c2.x);
                    angle_in_arc(a1, *sa1, *ea1) && angle_in_arc(a2, *sa2, *ea2)
                })
                .collect()
        }
    }
}

fn line_line_intersect(a1: &Pt2, a2: &Pt2, b1: &Pt2, b2: &Pt2) -> Vec<Pt2> {
    let d1x = a2.x - a1.x;
    let d1y = a2.y - a1.y;
    let d2x = b2.x - b1.x;
    let d2y = b2.y - b1.y;
    let denom = d1x * d2y - d1y * d2x;
    if denom.abs() < 1e-12 {
        return vec![]; // parallel or coincident
    }
    let t = ((b1.x - a1.x) * d2y - (b1.y - a1.y) * d2x) / denom;
    let u = ((b1.x - a1.x) * d1y - (b1.y - a1.y) * d1x) / denom;
    if t >= -1e-9 && t <= 1.0 + 1e-9 && u >= -1e-9 && u <= 1.0 + 1e-9 {
        vec![Pt2::new(a1.x + t * d1x, a1.y + t * d1y)]
    } else {
        vec![]
    }
}

fn line_circle_intersect(p1: &Pt2, p2: &Pt2, center: &Pt2, radius: f64) -> Vec<Pt2> {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    let fx = p1.x - center.x;
    let fy = p1.y - center.y;
    let a = dx * dx + dy * dy;
    let b = 2.0 * (fx * dx + fy * dy);
    let c = fx * fx + fy * fy - radius * radius;
    let disc = b * b - 4.0 * a * c;
    if disc < -1e-9 || a.abs() < 1e-18 {
        return vec![];
    }
    let disc = disc.max(0.0).sqrt();
    let mut results = Vec::new();
    for t in [(-b - disc) / (2.0 * a), (-b + disc) / (2.0 * a)] {
        if t >= -1e-9 && t <= 1.0 + 1e-9 {
            results.push(Pt2::new(p1.x + t * dx, p1.y + t * dy));
        }
    }
    results
}

fn line_arc_intersect(
    p1: &Pt2,
    p2: &Pt2,
    center: &Pt2,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
) -> Vec<Pt2> {
    let circle_pts = line_circle_intersect(p1, p2, center, radius);
    circle_pts
        .into_iter()
        .filter(|p| {
            let angle = (p.y - center.y).atan2(p.x - center.x);
            angle_in_arc(angle, start_angle, end_angle)
        })
        .collect()
}

fn circle_circle_intersect(c1: &Pt2, r1: f64, c2: &Pt2, r2: f64) -> Vec<Pt2> {
    let dx = c2.x - c1.x;
    let dy = c2.y - c1.y;
    let d = (dx * dx + dy * dy).sqrt();
    if d > r1 + r2 + 1e-9 || d < (r1 - r2).abs() - 1e-9 || d < 1e-12 {
        return vec![];
    }
    let a = (r1 * r1 - r2 * r2 + d * d) / (2.0 * d);
    let h_sq = r1 * r1 - a * a;
    let h = if h_sq < 0.0 { 0.0 } else { h_sq.sqrt() };
    let mx = c1.x + a * dx / d;
    let my = c1.y + a * dy / d;
    if h < 1e-12 {
        vec![Pt2::new(mx, my)]
    } else {
        vec![
            Pt2::new(mx + h * dy / d, my - h * dx / d),
            Pt2::new(mx - h * dy / d, my + h * dx / d),
        ]
    }
}

// ---------------------------------------------------------------------------
// Trim implementations
// ---------------------------------------------------------------------------

fn remove_entity_and_points(
    sketch: &mut Sketch,
    id: SketchEntityId,
    entity: &SketchEntity,
) -> KernelResult<()> {
    // Remove the entity itself.
    sketch.entities.remove(id)?;
    // Note: we intentionally do NOT remove referenced point entities here
    // because they may be shared with other entities.
    let _ = entity; // suppress unused warning
    Ok(())
}

fn trim_line(
    sketch: &mut Sketch,
    target_id: SketchEntityId,
    start: &Pt2,
    end: &Pt2,
    params: &[f64],
    click_t: f64,
) -> KernelResult<TrimResult> {
    // Build a sorted list of boundary parameters including 0 and 1.
    let mut boundaries = vec![0.0];
    boundaries.extend(params.iter().copied().filter(|&t| t > 1e-9 && t < 1.0 - 1e-9));
    boundaries.push(1.0);
    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());
    boundaries.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    // Find which segment the click falls in.
    let seg_idx = find_segment_index(&boundaries, click_t);

    // Remove the original line entity and its points.
    let original_entity = sketch.entities.get(target_id)?.clone();
    let (orig_start_id, orig_end_id) = match &original_entity {
        SketchEntity::Line { start, end } => (*start, *end),
        _ => unreachable!(),
    };
    sketch.entities.remove(target_id)?;

    let mut removed = vec![target_id];
    let mut added = Vec::new();

    // Re-create the segments that are NOT the clicked one.
    for i in 0..boundaries.len() - 1 {
        if i == seg_idx {
            continue;
        }
        let t0 = boundaries[i];
        let t1 = boundaries[i + 1];
        let sp = Pt2::new(
            start.x + t0 * (end.x - start.x),
            start.y + t0 * (end.y - start.y),
        );
        let ep = Pt2::new(
            start.x + t1 * (end.x - start.x),
            start.y + t1 * (end.y - start.y),
        );

        // Reuse original point IDs for endpoints that match original start/end.
        let sp_id = if t0.abs() < 1e-9 {
            orig_start_id
        } else {
            sketch.add_entity(SketchEntity::Point { position: sp })
        };
        let ep_id = if (t1 - 1.0).abs() < 1e-9 {
            orig_end_id
        } else {
            sketch.add_entity(SketchEntity::Point { position: ep })
        };

        let new_line = sketch.add_entity(SketchEntity::Line {
            start: sp_id,
            end: ep_id,
        });
        added.push(sp_id);
        added.push(ep_id);
        added.push(new_line);
    }

    Ok(TrimResult {
        removed_entities: removed,
        added_entities: added,
        modified_entities: vec![],
    })
}

fn trim_arc(
    sketch: &mut Sketch,
    target_id: SketchEntityId,
    center: &Pt2,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    params: &[f64],
    click_t: f64,
) -> KernelResult<TrimResult> {
    let span = normalize_angle(end_angle - start_angle);

    let mut boundaries = vec![0.0];
    boundaries.extend(params.iter().copied().filter(|&t| t > 1e-9 && t < 1.0 - 1e-9));
    boundaries.push(1.0);
    boundaries.sort_by(|a, b| a.partial_cmp(b).unwrap());
    boundaries.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    let seg_idx = find_segment_index(&boundaries, click_t);

    // Get original center point id.
    let original_entity = sketch.entities.get(target_id)?.clone();
    let orig_center_id = match &original_entity {
        SketchEntity::Arc { center, .. } => *center,
        _ => unreachable!(),
    };

    sketch.entities.remove(target_id)?;
    let mut removed = vec![target_id];
    let mut added = Vec::new();

    for i in 0..boundaries.len() - 1 {
        if i == seg_idx {
            continue;
        }
        let t0 = boundaries[i];
        let t1 = boundaries[i + 1];
        let sa = start_angle + t0 * span;
        let ea = start_angle + t1 * span;

        let sp = Pt2::new(center.x + radius * sa.cos(), center.y + radius * sa.sin());
        let ep = Pt2::new(center.x + radius * ea.cos(), center.y + radius * ea.sin());

        let sp_id = sketch.add_entity(SketchEntity::Point { position: sp });
        let ep_id = sketch.add_entity(SketchEntity::Point { position: ep });
        let new_arc = sketch.add_entity(SketchEntity::Arc {
            center: orig_center_id,
            start: sp_id,
            end: ep_id,
        });
        added.push(sp_id);
        added.push(ep_id);
        added.push(new_arc);
    }

    Ok(TrimResult {
        removed_entities: removed,
        added_entities: added,
        modified_entities: vec![],
    })
}

fn trim_circle(
    sketch: &mut Sketch,
    target_id: SketchEntityId,
    center: &Pt2,
    radius: f64,
    params: &[f64],
    click_t: f64,
) -> KernelResult<TrimResult> {
    // params are angles in [0, 2pi) for circles.
    let two_pi = 2.0 * std::f64::consts::PI;

    let mut angles: Vec<f64> = params.iter().copied().collect();
    angles.sort_by(|a, b| a.partial_cmp(b).unwrap());
    angles.dedup_by(|a, b| (*a - *b).abs() < 1e-9);

    if angles.len() < 2 {
        // With only one intersection, just remove the whole circle.
        sketch.entities.remove(target_id)?;
        return Ok(TrimResult {
            removed_entities: vec![target_id],
            added_entities: vec![],
            modified_entities: vec![],
        });
    }

    // Find which arc segment the click falls in.
    // Segments are defined between consecutive intersection angles (wrapping).
    let n = angles.len();
    let mut seg_idx = n - 1; // default: last segment (wrapping)
    for i in 0..n {
        let a_start = angles[i];
        let a_end = if i + 1 < n { angles[i + 1] } else { angles[0] + two_pi };
        if click_t >= a_start - 1e-9 && click_t <= a_end + 1e-9 {
            seg_idx = i;
            break;
        }
    }

    // Get original center point id.
    let original_entity = sketch.entities.get(target_id)?.clone();
    let orig_center_id = match &original_entity {
        SketchEntity::Circle { center, .. } => *center,
        _ => unreachable!(),
    };

    sketch.entities.remove(target_id)?;
    let mut removed = vec![target_id];
    let mut added = Vec::new();

    // Create arcs for all segments except the clicked one.
    for i in 0..n {
        if i == seg_idx {
            continue;
        }
        let sa = angles[i];
        let ea = if i + 1 < n { angles[i + 1] } else { angles[0] + two_pi };

        let sp = Pt2::new(center.x + radius * sa.cos(), center.y + radius * sa.sin());
        let ep = Pt2::new(center.x + radius * ea.cos(), center.y + radius * ea.sin());

        let sp_id = sketch.add_entity(SketchEntity::Point { position: sp });
        let ep_id = sketch.add_entity(SketchEntity::Point { position: ep });
        let new_arc = sketch.add_entity(SketchEntity::Arc {
            center: orig_center_id,
            start: sp_id,
            end: ep_id,
        });
        added.push(sp_id);
        added.push(ep_id);
        added.push(new_arc);
    }

    Ok(TrimResult {
        removed_entities: removed,
        added_entities: added,
        modified_entities: vec![],
    })
}

/// Find which segment (between sorted boundaries) a parameter value falls in.
fn find_segment_index(boundaries: &[f64], t: f64) -> usize {
    for i in 0..boundaries.len() - 1 {
        if t >= boundaries[i] - 1e-9 && t <= boundaries[i + 1] + 1e-9 {
            return i;
        }
    }
    // Fallback: nearest segment midpoint.
    let mut best = 0;
    let mut best_dist = f64::MAX;
    for i in 0..boundaries.len() - 1 {
        let mid = (boundaries[i] + boundaries[i + 1]) / 2.0;
        let dist = (t - mid).abs();
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    best
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::sketch::sketch::Sketch;

    /// Helper: create a sketch with two crossing lines forming an X pattern.
    /// Line1: (-1,0) -> (1,0)  (horizontal)
    /// Line2: (0,-1) -> (0,1)  (vertical)
    fn make_cross_sketch() -> Sketch {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(-1.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(1.0, 0.0) });
        let _line1 = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, -1.0) });
        let p4 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 1.0) });
        let _line2 = sketch.add_entity(SketchEntity::Line { start: p3, end: p4 });
        sketch
    }

    #[test]
    fn trim_line_between_two_intersecting_lines() {
        let mut sketch = make_cross_sketch();
        // Click on the right half of the horizontal line (x=0.5, y=0).
        // The intersection is at (0,0), so the right segment [0,0]->[1,0]
        // should be removed, and the left segment [-1,0]->[0,0] kept.
        let result = trim_entity(&mut sketch, Pt2::new(0.5, 0.0), 0.5).unwrap();

        assert_eq!(result.removed_entities.len(), 1);
        // Should have added one new line segment (left part) with its points.
        assert!(!result.added_entities.is_empty());

        // Verify the remaining line segment endpoints: should have a line
        // from (-1,0) to approximately (0,0).
        let lines: Vec<_> = sketch
            .entities
            .iter()
            .filter_map(|(_, e)| match e {
                SketchEntity::Line { start, end } => Some((*start, *end)),
                _ => None,
            })
            .collect();

        // Should have the original vertical line plus one remaining horizontal segment.
        // (The vertical line is untouched, the horizontal line was split.)
        assert_eq!(lines.len(), 2); // vertical line + left segment

        // Verify one of the new lines has endpoints near (-1,0) and (0,0).
        let mut found_left_segment = false;
        for (s, e) in &lines {
            let sp = get_point_position(&sketch, *s).unwrap();
            let ep = get_point_position(&sketch, *e).unwrap();
            if (sp.x - (-1.0)).abs() < 0.01 && sp.y.abs() < 0.01
                && ep.x.abs() < 0.01 && ep.y.abs() < 0.01
            {
                found_left_segment = true;
            }
        }
        assert!(found_left_segment, "Expected left segment [-1,0]->[0,0] to remain");
    }

    #[test]
    fn trim_with_no_intersections_removes_entity() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(1.0, 0.0) });
        let line_id = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        // No other entities to intersect with.
        let result = trim_entity(&mut sketch, Pt2::new(0.5, 0.0), 0.5).unwrap();
        assert_eq!(result.removed_entities, vec![line_id]);
        assert!(result.added_entities.is_empty());

        // The line should be gone.
        assert!(sketch.entities.get(line_id).is_err());
    }

    #[test]
    fn trim_circle_converts_to_arc() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Circle centered at origin, radius 1.
        let center = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let circle_id = sketch.add_entity(SketchEntity::Circle { center, radius: 1.0 });

        // Add two lines that cross the circle.
        // Horizontal line through center.
        let lp1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(-2.0, 0.0) });
        let lp2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 0.0) });
        let _hline = sketch.add_entity(SketchEntity::Line { start: lp1, end: lp2 });

        // Click on the top of the circle (0, 1).
        // The line intersects the circle at (-1,0) and (1,0).
        // The top arc should be removed, bottom arc kept.
        let result = trim_entity(&mut sketch, Pt2::new(0.0, 1.0), 0.5).unwrap();

        assert_eq!(result.removed_entities.len(), 1);
        assert_eq!(result.removed_entities[0], circle_id);

        // Should have created at least one arc.
        let arcs: Vec<_> = sketch
            .entities
            .iter()
            .filter_map(|(id, e)| match e {
                SketchEntity::Arc { .. } => Some(id),
                _ => None,
            })
            .collect();
        assert!(!arcs.is_empty(), "Expected at least one arc after trimming circle");
    }

    #[test]
    fn trim_arc_between_intersections() {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        // Create a semicircular arc (top half of unit circle).
        let center = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let arc_start = sketch.add_entity(SketchEntity::Point { position: Pt2::new(1.0, 0.0) });
        let arc_end = sketch.add_entity(SketchEntity::Point { position: Pt2::new(-1.0, 0.0) });
        let arc_id = sketch.add_entity(SketchEntity::Arc {
            center,
            start: arc_start,
            end: arc_end,
        });

        // Vertical line from (0, -1) to (0, 2) crossing the arc at (0, 1).
        let vp1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, -1.0) });
        let vp2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 2.0) });
        let _vline = sketch.add_entity(SketchEntity::Line { start: vp1, end: vp2 });

        // Click on the right part of the arc, near (0.7, 0.7).
        let result = trim_entity(&mut sketch, Pt2::new(0.7, 0.7), 0.5).unwrap();
        assert_eq!(result.removed_entities.len(), 1);
        assert_eq!(result.removed_entities[0], arc_id);
        // Should have added remaining arc segment(s).
        assert!(!result.added_entities.is_empty());
    }

    #[test]
    fn trim_no_entity_near_click_returns_error() {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(1.0, 0.0) });
        let _line = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });

        // Click far away from the line.
        let result = trim_entity(&mut sketch, Pt2::new(100.0, 100.0), 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn trim_line_left_segment() {
        let mut sketch = make_cross_sketch();
        // Click on the left half of the horizontal line (x=-0.5, y=0).
        let result = trim_entity(&mut sketch, Pt2::new(-0.5, 0.0), 0.5).unwrap();
        assert_eq!(result.removed_entities.len(), 1);

        // The right segment should remain.
        let mut found_right = false;
        for (_, e) in sketch.entities.iter() {
            if let SketchEntity::Line { start, end } = e {
                let sp = get_point_position(&sketch, *start).unwrap();
                let ep = get_point_position(&sketch, *end).unwrap();
                if sp.x.abs() < 0.01 && sp.y.abs() < 0.01
                    && (ep.x - 1.0).abs() < 0.01 && ep.y.abs() < 0.01
                {
                    found_right = true;
                }
            }
        }
        assert!(found_right, "Expected right segment [0,0]->[1,0] to remain");
    }
}
