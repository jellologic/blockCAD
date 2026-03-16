use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::operations::cut_extrude::cut_extrude;
use crate::operations::extrude::{extrude_profile, EndCondition, ExtrudeProfile, FromCondition};
use crate::operations::revolve::revolve_profile;
use crate::sketch::profile::extract_profile;
use crate::sketch::solver_bridge::build_constraint_graph;
use crate::solver::newton_raphson::{solve, SolverConfig};
use crate::topology::body::Body;
use crate::topology::BRep;

use super::feature::FeatureState;
use super::kind::FeatureKind;
use super::params::FeatureParams;
use super::tree::FeatureTree;

/// Cast a ray from the profile centroid along the extrude direction
/// and find the nearest face of the existing BRep.
fn compute_up_to_next_depth(
    brep: &BRep,
    profile: &ExtrudeProfile,
    direction: Vec3,
) -> Option<f64> {
    if brep.faces.is_empty() {
        return None;
    }
    let n = profile.points.len() as f64;
    let centroid = {
        let sum = profile.points.iter().fold(
            Pt3::new(0.0, 0.0, 0.0),
            |acc, p| Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
        );
        Pt3::new(sum.x / n, sum.y / n, sum.z / n)
    };
    let dir = direction.normalize();

    let mut min_t = f64::INFINITY;
    for (_, face) in brep.faces.iter() {
        if let Some(surf_idx) = face.surface_index {
            if let (Ok(normal), Ok(origin)) = (
                brep.surfaces[surf_idx].normal_at(0.0, 0.0),
                brep.surfaces[surf_idx].point_at(0.0, 0.0),
            ) {
                let denom = dir.dot(&normal);
                if denom.abs() > 1e-12 {
                    let diff = origin - centroid;
                    let t = Vec3::new(diff.x, diff.y, diff.z).dot(&normal) / denom;
                    if t > 1e-6 && t < min_t {
                        min_t = t;
                    }
                }
            }
        }
    }
    if min_t.is_finite() {
        Some(min_t)
    } else {
        None
    }
}

/// Cast a ray from the profile centroid along the extrude direction
/// and find the depth to a specific face of the existing BRep.
fn compute_up_to_surface_depth(
    brep: &BRep,
    profile: &ExtrudeProfile,
    direction: Vec3,
    face_index: usize,
) -> Option<f64> {
    // Get the N-th face (iteration order matches tessellation)
    let face = brep.faces.iter().nth(face_index)?;
    let (_, face) = face;
    let surf_idx = face.surface_index?;
    let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0).ok()?;
    let origin = brep.surfaces[surf_idx].point_at(0.0, 0.0).ok()?;

    let n = profile.points.len() as f64;
    let centroid = {
        let sum = profile.points.iter().fold(
            Pt3::new(0.0, 0.0, 0.0),
            |acc, p| Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
        );
        Pt3::new(sum.x / n, sum.y / n, sum.z / n)
    };
    let dir = direction.normalize();

    let denom = dir.dot(&normal);
    if denom.abs() <= 1e-12 {
        return None; // parallel
    }
    let diff = origin - centroid;
    let t = Vec3::new(diff.x, diff.y, diff.z).dot(&normal) / denom;
    if t > 1e-6 { Some(t) } else { None }
}

/// Cast a ray from the profile centroid along the extrude direction
/// and find the signed distance to a specific face of the existing BRep.
/// Unlike compute_up_to_surface_depth, this allows negative t (face behind sketch plane).
fn compute_from_surface_offset(
    brep: &BRep,
    profile: &ExtrudeProfile,
    direction: Vec3,
    face_index: usize,
) -> Option<f64> {
    let face_data = brep.faces.iter().nth(face_index)?;
    let (_, face) = face_data;
    let surf_idx = face.surface_index?;
    let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0).ok()?;
    let origin = brep.surfaces[surf_idx].point_at(0.0, 0.0).ok()?;

    let n = profile.points.len() as f64;
    let centroid = {
        let sum = profile.points.iter().fold(
            Pt3::new(0.0, 0.0, 0.0),
            |acc, p| Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
        );
        Pt3::new(sum.x / n, sum.y / n, sum.z / n)
    };
    let dir = direction.normalize();
    let denom = dir.dot(&normal);
    if denom.abs() <= 1e-12 {
        return None;
    }
    let diff = origin - centroid;
    let t = Vec3::new(diff.x, diff.y, diff.z).dot(&normal) / denom;
    Some(t) // can be negative (face behind sketch plane)
}

/// Evaluate the feature tree, producing the final BRep.
///
/// Replays operations from scratch up to the cursor.
/// Features marked as Suppressed are skipped.
///
/// Currently rebuilds from scratch each time (no incremental caching).
/// Caching will be added once BRep supports Clone or we implement
/// a copy-on-write scheme.
pub fn evaluate(tree: &mut FeatureTree) -> KernelResult<BRep> {
    let cursor = match tree.cursor() {
        Some(c) => c,
        None => return Ok(BRep::new()),
    };

    let mut current_brep = BRep::new();

    for i in 0..=cursor {
        let feature = &tree.features()[i];

        if feature.suppressed {
            tree.features_mut()[i].state = FeatureState::Evaluated;
            continue;
        }

        match feature.kind {
            FeatureKind::Sketch => {
                // Get sketch: prefer from params, fall back to tree.sketches side-channel
                let sketch = if let FeatureParams::Sketch(s) = &tree.features()[i].params {
                    s.clone()
                } else if let Some(s) = tree.sketches.get(&i) {
                    s.clone()
                } else {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: format!("No sketch data for feature at index {}", i),
                    });
                };
                // Populate the side-channel for profile extraction
                tree.sketches.insert(i, sketch.clone());

                let (mut graph, var_map) = build_constraint_graph(&sketch)?;
                let result = solve(&mut graph, &SolverConfig::default())?;
                if !result.converged {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::ConstraintSolver {
                        reason: "Sketch solver did not converge".into(),
                        dof: None,
                    });
                }

                let profile = extract_profile(&sketch, &var_map, &graph)?;
                tree.sketch_profiles.insert(i, profile);
                tree.features_mut()[i].state = FeatureState::Evaluated;
                // Sketch doesn't produce geometry — current_brep unchanged
            }

            FeatureKind::Extrude => {
                let mut params = match &tree.features()[i].params {
                    FeatureParams::Extrude(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Extrude feature has wrong params type".into(),
                        });
                    }
                };

                let profile = find_latest_sketch_profile(tree, i)?;

                // Pre-compute depth for UpToNext
                if params.end_condition == EndCondition::UpToNext {
                    if let Some(depth) =
                        compute_up_to_next_depth(&current_brep, &profile, params.direction)
                    {
                        params.up_to_next_depth = Some(depth);
                    }
                }

                // Pre-compute depth for UpToSurface / OffsetFromSurface
                if matches!(params.end_condition, EndCondition::UpToSurface | EndCondition::OffsetFromSurface) {
                    if let Some(face_idx) = params.target_face_index {
                        if let Some(depth) = compute_up_to_surface_depth(
                            &current_brep, &profile, params.direction, face_idx,
                        ) {
                            params.up_to_next_depth = Some(depth);
                        }
                    }
                }

                // Handle From: Surface
                if params.from_condition == FromCondition::Surface {
                    if let Some(face_idx) = params.from_face_index {
                        if let Some(offset) = compute_from_surface_offset(&current_brep, &profile, params.direction, face_idx) {
                            params.from_offset = offset;
                        }
                    }
                }
                // Handle From: Vertex
                if params.from_condition == FromCondition::Vertex {
                    if let Some(pos) = params.from_vertex_position {
                        let centroid = {
                            let n = profile.points.len() as f64;
                            let sum = profile.points.iter().fold(
                                Pt3::new(0.0, 0.0, 0.0),
                                |acc, p| Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
                            );
                            Pt3::new(sum.x / n, sum.y / n, sum.z / n)
                        };
                        let dir = params.direction.normalize();
                        let v = Pt3::new(pos[0], pos[1], pos[2]);
                        let diff = v - centroid;
                        params.from_offset = Vec3::new(diff.x, diff.y, diff.z).dot(&dir);
                    }
                }

                current_brep = extrude_profile(&profile, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::CutExtrude => {
                let mut params = match &tree.features()[i].params {
                    FeatureParams::CutExtrude(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "CutExtrude feature has wrong params type".into(),
                        });
                    }
                };

                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot cut: no existing geometry to cut from".into(),
                    });
                }

                let profile = find_latest_sketch_profile(tree, i)?;

                // Pre-compute depth for UpToNext
                if params.end_condition == EndCondition::UpToNext {
                    if let Some(depth) =
                        compute_up_to_next_depth(&current_brep, &profile, params.direction)
                    {
                        params.up_to_next_depth = Some(depth);
                    }
                }

                // Pre-compute depth for UpToSurface / OffsetFromSurface
                if matches!(params.end_condition, EndCondition::UpToSurface | EndCondition::OffsetFromSurface) {
                    if let Some(face_idx) = params.target_face_index {
                        if let Some(depth) = compute_up_to_surface_depth(
                            &current_brep, &profile, params.direction, face_idx,
                        ) {
                            params.up_to_next_depth = Some(depth);
                        }
                    }
                }

                // Handle From: Surface
                if params.from_condition == FromCondition::Surface {
                    if let Some(face_idx) = params.from_face_index {
                        if let Some(offset) = compute_from_surface_offset(&current_brep, &profile, params.direction, face_idx) {
                            params.from_offset = offset;
                        }
                    }
                }
                // Handle From: Vertex
                if params.from_condition == FromCondition::Vertex {
                    if let Some(pos) = params.from_vertex_position {
                        let centroid = {
                            let n = profile.points.len() as f64;
                            let sum = profile.points.iter().fold(
                                Pt3::new(0.0, 0.0, 0.0),
                                |acc, p| Pt3::new(acc.x + p.x, acc.y + p.y, acc.z + p.z),
                            );
                            Pt3::new(sum.x / n, sum.y / n, sum.z / n)
                        };
                        let dir = params.direction.normalize();
                        let v = Pt3::new(pos[0], pos[1], pos[2]);
                        let diff = v - centroid;
                        params.from_offset = Vec3::new(diff.x, diff.y, diff.z).dot(&dir);
                    }
                }

                current_brep = cut_extrude(current_brep, &profile, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Revolve => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Revolve(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Revolve feature has wrong params type".into(),
                        });
                    }
                };

                let profile = find_latest_sketch_profile(tree, i)?;
                current_brep = revolve_profile(&profile, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::CutRevolve => {
                let params = match &tree.features()[i].params {
                    FeatureParams::CutRevolve(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "CutRevolve feature has wrong params type".into(),
                        });
                    }
                };

                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot cut: no existing geometry to cut from".into(),
                    });
                }

                let profile = find_latest_sketch_profile(tree, i)?;
                // NOTE: CutRevolve currently creates additive geometry (same as boss revolve).
                // Proper boolean subtract for curved geometry requires a CSG engine which is
                // not yet implemented. The UI warns the user about this limitation.
                current_brep = revolve_profile(&profile, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Chamfer => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Chamfer(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Chamfer feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot chamfer: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::chamfer::chamfer_edges(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Fillet => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Fillet(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Fillet feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot fillet: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::fillet::fillet_edges(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::LinearPattern => {
                let params = match &tree.features()[i].params {
                    FeatureParams::LinearPattern(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "LinearPattern feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot pattern: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::pattern::linear::linear_pattern(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::CircularPattern => {
                let params = match &tree.features()[i].params {
                    FeatureParams::CircularPattern(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "CircularPattern feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot pattern: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::pattern::circular::circular_pattern(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Mirror => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Mirror(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Mirror feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot mirror: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::pattern::mirror::mirror_brep(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Shell => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Shell(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Shell feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot shell: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::shell::shell_solid(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Draft => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Draft(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Draft feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot draft: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::draft::draft_faces(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::DatumPlane => {
                let params = match &tree.features()[i].params {
                    FeatureParams::DatumPlane(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "DatumPlane feature has wrong params type".into(),
                        });
                    }
                };
                // Resolve base plane: check datum_planes registry or use XY default
                let base_plane = params.base_plane_index
                    .and_then(|idx| tree.datum_planes.get(&idx))
                    .cloned()
                    .unwrap_or_else(|| crate::geometry::surface::plane::Plane::xy(0.0));

                let brep_ref = if matches!(current_brep.body, Body::Empty) { None } else { Some(&current_brep) };
                let plane = crate::operations::datum_plane::compute_datum_plane(
                    &params.kind, Some(&base_plane), brep_ref,
                )?;
                tree.datum_planes.insert(i, plane);
                // Datum planes don't produce geometry — current_brep unchanged
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            other => {
                tree.features_mut()[i].state = FeatureState::Failed;
                return Err(KernelError::Operation {
                    op: "evaluate".into(),
                    detail: format!("{:?} operation not yet implemented", other),
                });
            }
        }
    }

    Ok(current_brep)
}

/// Find the most recent solved sketch profile before the given feature index.
fn find_latest_sketch_profile(
    tree: &FeatureTree,
    before_index: usize,
) -> KernelResult<crate::operations::extrude::ExtrudeProfile> {
    for i in (0..before_index).rev() {
        if tree.features()[i].kind == FeatureKind::Sketch && !tree.features()[i].suppressed {
            if let Some(profile) = tree.sketch_profiles.get(&i) {
                return Ok(profile.clone());
            }
        }
    }
    Err(KernelError::Operation {
        op: "evaluate".into(),
        detail: "No sketch profile found before extrude feature".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_tree::feature::Feature;
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Pt2, Vec3};
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::Sketch;
    use crate::topology::body::Body;

    fn make_rectangle_sketch() -> Sketch {
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let p0 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.0, 0.0),
        });
        let p1 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(8.0, 0.5),
        });
        let p2 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(8.0, 4.0),
        });
        let p3 = sketch.add_entity(SketchEntity::Point {
            position: Pt2::new(0.5, 4.0),
        });

        let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 10.0 },
            vec![p0, p1],
        ));
        sketch.add_constraint(Constraint::new(
            ConstraintKind::Distance { value: 5.0 },
            vec![p1, p2],
        ));

        sketch
    }

    fn build_sketch_extrude_tree(depth: f64) -> FeatureTree {
        let mut tree = FeatureTree::new();

        tree.push(Feature::new(
            "sketch-1".into(),
            "Base Sketch".into(),
            FeatureKind::Sketch,
            FeatureParams::Placeholder,
        ));
        tree.sketches.insert(0, make_rectangle_sketch());

        tree.push(Feature::new(
            "extrude-1".into(),
            "Extrude Base".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams::blind(
                Vec3::new(0.0, 0.0, 1.0),
                depth,
            )),
        ));

        tree
    }

    #[test]
    fn test_empty_tree_returns_empty_brep() {
        let mut tree = FeatureTree::new();
        let brep = evaluate(&mut tree).unwrap();
        assert!(matches!(brep.body, Body::Empty));
    }

    #[test]
    fn test_sketch_then_extrude_produces_6_faces() {
        let mut tree = build_sketch_extrude_tree(7.0);
        let brep = evaluate(&mut tree).unwrap();

        assert_eq!(brep.faces.len(), 6, "Extruded rectangle should have 6 faces");
        assert!(matches!(brep.body, Body::Solid(_)));
    }

    #[test]
    fn test_suppress_extrude_returns_empty() {
        let mut tree = build_sketch_extrude_tree(7.0);
        tree.suppress(1).unwrap();

        let brep = evaluate(&mut tree).unwrap();
        assert!(matches!(brep.body, Body::Empty));
    }

    #[test]
    fn test_evaluated_state_set() {
        let mut tree = build_sketch_extrude_tree(7.0);
        evaluate(&mut tree).unwrap();

        assert_eq!(tree.features()[0].state, FeatureState::Evaluated);
        assert_eq!(tree.features()[1].state, FeatureState::Evaluated);
    }

    #[test]
    fn test_rollback_truncates() {
        let mut tree = build_sketch_extrude_tree(7.0);

        evaluate(&mut tree).unwrap();

        // Roll back to before extrude
        tree.rollback_to(1).unwrap();
        let brep = evaluate(&mut tree).unwrap();
        assert!(matches!(brep.body, Body::Empty));

        // Roll forward
        tree.roll_forward();
        let brep = evaluate(&mut tree).unwrap();
        assert_eq!(brep.faces.len(), 6);
    }

    #[test]
    fn test_unsuppress_recomputes() {
        let mut tree = build_sketch_extrude_tree(7.0);

        tree.suppress(1).unwrap();
        let brep = evaluate(&mut tree).unwrap();
        assert!(matches!(brep.body, Body::Empty));

        tree.unsuppress(1).unwrap();
        let brep = evaluate(&mut tree).unwrap();
        assert_eq!(brep.faces.len(), 6);
    }

    #[test]
    fn test_extrude_without_sketch_fails() {
        let mut tree = FeatureTree::new();
        tree.push(Feature::new(
            "extrude-1".into(),
            "Extrude".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams::blind(
                Vec3::new(0.0, 0.0, 1.0),
                5.0,
            )),
        ));

        let result = evaluate(&mut tree);
        assert!(result.is_err(), "Extrude without sketch should fail");
    }
}
