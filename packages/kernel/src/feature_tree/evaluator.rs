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
/// Uses cached BRep snapshots to skip unchanged features. Only features
/// from the first dirty (uncached) index onward are re-evaluated.
/// Features marked as Suppressed are skipped.
pub fn evaluate(tree: &mut FeatureTree) -> KernelResult<BRep> {
    let cursor = match tree.cursor() {
        Some(c) => c,
        None => return Ok(BRep::new()),
    };

    // Find the latest valid cache entry before any dirty feature.
    // A feature is "dirty" if its cache slot is None.
    let start_from = (0..=cursor)
        .find(|&i| tree.cache_at(i).is_none())
        .unwrap_or(cursor + 1); // all cached

    // If everything up to cursor is cached, return the cached result directly.
    if start_from > cursor {
        return Ok(tree.cache_at(cursor).unwrap().clone());
    }

    // Load BRep state from the cache entry just before the first dirty feature.
    let mut current_brep = if start_from > 0 {
        tree.cache_at(start_from - 1).unwrap().clone()
    } else {
        BRep::new()
    };

    for i in start_from..=cursor {
        let feature = &tree.features()[i];

        if feature.suppressed {
            tree.features_mut()[i].state = FeatureState::Evaluated;
            tree.set_cache(i, current_brep.clone());
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

            FeatureKind::VariableFillet => {
                let params = match &tree.features()[i].params {
                    FeatureParams::VariableFillet(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "VariableFillet feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot variable fillet: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::fillet::variable_fillet_edges(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::FaceFillet => {
                let params = match &tree.features()[i].params {
                    FeatureParams::FaceFillet(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "FaceFillet feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot face fillet: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::fillet::face_fillet(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::MoveBody => {
                let params = match &tree.features()[i].params {
                    FeatureParams::MoveBody(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "MoveBody feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot move/copy: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::transform_body::move_body(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::ScaleBody => {
                let params = match &tree.features()[i].params {
                    FeatureParams::ScaleBody(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "ScaleBody feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot scale: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::transform::scale_body(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::HoleWizard => {
                let params = match &tree.features()[i].params {
                    FeatureParams::HoleWizard(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "HoleWizard feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot create hole: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::hole::hole_wizard(current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Dome => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Dome(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Dome feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot dome: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::dome::dome_face(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::Rib => {
                let params = match &tree.features()[i].params {
                    FeatureParams::Rib(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "Rib feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot rib: no existing geometry".into(),
                    });
                }
                let profile = find_latest_sketch_profile(tree, i)?;
                current_brep = crate::operations::rib::rib_from_profile(&current_brep, &profile, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::SplitBody => {
                let params = match &tree.features()[i].params {
                    FeatureParams::SplitBody(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "SplitBody feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot split: no existing geometry".into(),
                    });
                }
                current_brep = crate::operations::boolean::split::split_body(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::CombineBodies => {
                let params = match &tree.features()[i].params {
                    FeatureParams::CombineBodies(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "CombineBodies feature has wrong params type".into(),
                        });
                    }
                };
                if matches!(current_brep.body, Body::Empty) {
                    tree.features_mut()[i].state = FeatureState::Failed;
                    return Err(KernelError::Operation {
                        op: "evaluate".into(),
                        detail: "Cannot combine: no existing geometry".into(),
                    });
                }
                let tool_body = match tree.tool_bodies.remove(&i) {
                    Some(b) => b,
                    None => BRep::new(),
                };
                current_brep = crate::operations::boolean::combine::combine_bodies(&current_brep, &tool_body, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::CurvePattern => {
                let params = match &tree.features()[i].params {
                    FeatureParams::CurvePattern(p) => p.clone(),
                    _ => {
                        tree.features_mut()[i].state = FeatureState::Failed;
                        return Err(KernelError::Operation {
                            op: "evaluate".into(),
                            detail: "CurvePattern feature has wrong params type".into(),
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
                current_brep = crate::operations::pattern::curve::curve_pattern(&current_brep, &params)?;
                tree.features_mut()[i].state = FeatureState::Evaluated;
            }

            FeatureKind::ReferenceAxis | FeatureKind::ReferencePoint | FeatureKind::CoordinateSystem => {
                // Reference geometry doesn't modify the BRep
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

        // Cache the BRep state after this feature for incremental re-evaluation.
        tree.set_cache(i, current_brep.clone());
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

    /// Build a tree with sketch(0) + extrude(1) + chamfer(2).
    fn build_three_feature_tree() -> FeatureTree {
        let mut tree = build_sketch_extrude_tree(7.0);
        tree.push(Feature::new(
            "chamfer-1".into(),
            "Chamfer Edges".into(),
            FeatureKind::Chamfer,
            FeatureParams::Chamfer(crate::operations::chamfer::ChamferParams {
                edge_indices: vec![0],
                distance: 0.5,
                distance2: None,
                mode: None,
            }),
        ));
        tree
    }

    #[test]
    fn test_cache_populated_after_evaluate() {
        let mut tree = build_sketch_extrude_tree(7.0);
        evaluate(&mut tree).unwrap();

        // Both features should now have cached BRep state
        assert!(tree.cache_at(0).is_some(), "Sketch cache should be populated");
        assert!(tree.cache_at(1).is_some(), "Extrude cache should be populated");
    }

    #[test]
    fn test_cache_hit_skips_upstream_features() {
        let mut tree = build_three_feature_tree();

        // First evaluation populates all caches
        let brep1 = evaluate(&mut tree).unwrap();
        let face_count_1 = brep1.faces.len();

        // Invalidate only the last feature (chamfer)
        tree.invalidate_from(2);
        assert!(tree.cache_at(0).is_some(), "Sketch cache should survive");
        assert!(tree.cache_at(1).is_some(), "Extrude cache should survive");
        assert!(tree.cache_at(2).is_none(), "Chamfer cache should be cleared");

        // Re-evaluate: should reuse cache for features 0-1, only re-run feature 2
        let brep2 = evaluate(&mut tree).unwrap();
        assert_eq!(brep2.faces.len(), face_count_1, "Result should be identical");
        assert!(tree.cache_at(2).is_some(), "Chamfer cache should be repopulated");
    }

    #[test]
    fn test_cache_miss_when_early_feature_modified() {
        let mut tree = build_three_feature_tree();

        // First evaluation
        evaluate(&mut tree).unwrap();

        // Invalidate from the extrude (feature 1) -- simulates param change
        tree.invalidate_from(1);
        assert!(tree.cache_at(0).is_some(), "Sketch cache should survive");
        assert!(tree.cache_at(1).is_none(), "Extrude cache should be cleared");
        assert!(tree.cache_at(2).is_none(), "Chamfer cache should be cleared");

        // Re-evaluate: features 1 and 2 must be re-evaluated
        let brep = evaluate(&mut tree).unwrap();
        assert!(tree.cache_at(1).is_some(), "Extrude cache should be repopulated");
        assert!(tree.cache_at(2).is_some(), "Chamfer cache should be repopulated");
        assert!(brep.faces.len() > 0);
    }

    #[test]
    fn test_suppress_invalidates_downstream_cache() {
        let mut tree = build_three_feature_tree();

        // First evaluation
        evaluate(&mut tree).unwrap();
        assert!(tree.cache_at(2).is_some());

        // Suppress the extrude (feature 1) -- invalidates features 1+
        tree.suppress(1).unwrap();
        assert!(tree.cache_at(1).is_none(), "Suppressed feature cache should be cleared");
        assert!(tree.cache_at(2).is_none(), "Downstream cache should be cleared");

        // Chamfer will fail because there's no geometry, which is expected.
        // The extrude is suppressed so current_brep stays empty.
        let result = evaluate(&mut tree);
        assert!(result.is_err(), "Chamfer on empty body should fail");
    }

    #[test]
    fn test_push_new_feature_does_not_clear_existing_cache() {
        let mut tree = build_sketch_extrude_tree(7.0);
        evaluate(&mut tree).unwrap();

        assert!(tree.cache_at(0).is_some());
        assert!(tree.cache_at(1).is_some());

        // Push a new feature at the end
        tree.push(Feature::new(
            "chamfer-1".into(),
            "Chamfer".into(),
            FeatureKind::Chamfer,
            FeatureParams::Chamfer(crate::operations::chamfer::ChamferParams {
                edge_indices: vec![0],
                distance: 0.5,
                distance2: None,
                mode: None,
            }),
        ));

        // Existing caches should be preserved
        assert!(tree.cache_at(0).is_some(), "Existing sketch cache should survive push");
        assert!(tree.cache_at(1).is_some(), "Existing extrude cache should survive push");
        assert!(tree.cache_at(2).is_none(), "New feature should start uncached");

        // Re-evaluate: only the new feature should be computed
        let brep = evaluate(&mut tree).unwrap();
        assert!(tree.cache_at(2).is_some(), "New feature cache should be populated");
        assert!(brep.faces.len() > 0);
    }

    #[test]
    fn test_update_params_invalidates_cache() {
        let mut tree = build_sketch_extrude_tree(7.0);
        evaluate(&mut tree).unwrap();

        assert!(tree.cache_at(0).is_some());
        assert!(tree.cache_at(1).is_some());

        // Update extrude params (changes depth)
        tree.update_params(
            1,
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams::blind(
                Vec3::new(0.0, 0.0, 1.0),
                12.0,
            )),
        ).unwrap();

        assert!(tree.cache_at(0).is_some(), "Upstream cache should survive");
        assert!(tree.cache_at(1).is_none(), "Modified feature cache should be cleared");

        let brep = evaluate(&mut tree).unwrap();
        assert_eq!(brep.faces.len(), 6);
        assert!(tree.cache_at(1).is_some(), "Cache should be repopulated");
    }

    #[test]
    fn test_fully_cached_returns_without_reeval() {
        let mut tree = build_sketch_extrude_tree(7.0);

        // First evaluation
        let brep1 = evaluate(&mut tree).unwrap();

        // Second evaluation should hit cache entirely
        let brep2 = evaluate(&mut tree).unwrap();

        assert_eq!(brep1.faces.len(), brep2.faces.len());
        assert_eq!(brep1.vertices.len(), brep2.vertices.len());
        assert_eq!(brep1.edges.len(), brep2.edges.len());
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
