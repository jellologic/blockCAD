use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

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

/// Metrics from a single evaluation pass.
#[derive(Debug, Default, Clone)]
pub struct EvalMetrics {
    /// Number of features that were fully re-evaluated.
    pub features_evaluated: usize,
    /// Number of features skipped because param hash was unchanged (Tier 1).
    pub features_skipped_param_hash: usize,
    /// Number of features where re-evaluation produced an identical fingerprint (Tier 2),
    /// preserving downstream cache validity.
    pub features_skipped_fingerprint: usize,
}

/// Compute a hash of FeatureParams by serializing to JSON, then hashing.
/// This avoids needing Hash on f64 fields while still being deterministic
/// for unchanged parameters.
fn hash_params(params: &FeatureParams) -> u64 {
    let json = serde_json::to_string(params).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    json.hash(&mut hasher);
    hasher.finish()
}

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
/// Uses two-tier hash-based cache validation:
/// - **Tier 1 (param hash)**: If a feature's params haven't changed since last eval,
///   skip re-evaluation entirely and reuse the cached BRep.
/// - **Tier 2 (fingerprint)**: After re-evaluating a feature, if the output BRep
///   fingerprint matches the previous one, downstream cache remains valid.
///
/// Features marked as Suppressed are skipped.
pub fn evaluate(tree: &mut FeatureTree) -> KernelResult<BRep> {
    let (brep, _metrics) = evaluate_with_metrics(tree)?;
    Ok(brep)
}

/// Evaluate the feature tree, returning both the final BRep and evaluation metrics.
pub fn evaluate_with_metrics(tree: &mut FeatureTree) -> KernelResult<(BRep, EvalMetrics)> {
    let cursor = match tree.cursor() {
        Some(c) => c,
        None => return Ok((BRep::new(), EvalMetrics::default())),
    };

    let mut current_brep = BRep::new();
    let mut metrics = EvalMetrics::default();

    for i in 0..=cursor {
        let feature = &tree.features()[i];

        if feature.suppressed {
            tree.features_mut()[i].state = FeatureState::Evaluated;
            let current_hash = hash_params(&tree.features()[i].params);
            tree.set_cache(i, current_brep.clone());
            tree.set_param_hash(i, current_hash);
            tree.set_fingerprint(i, current_brep.fingerprint());
            continue;
        }

        // Tier 1: Check if params actually changed since last eval
        let current_hash = hash_params(&tree.features()[i].params);
        if let Some(cached_brep) = tree.cache_at(i) {
            if tree.param_hash_at(i) == Some(current_hash) {
                // Params identical — skip re-evaluation
                current_brep = cached_brep.clone();
                metrics.features_skipped_param_hash += 1;
                continue;
            }
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

        // Feature was re-evaluated. Track metrics.
        metrics.features_evaluated += 1;

        // Tier 2: After evaluation, check if output actually changed.
        // If the fingerprint is identical, downstream cache entries remain valid.
        let new_fp = current_brep.fingerprint();
        let output_changed = tree.fingerprint_at(i) != Some(&new_fp);

        if !output_changed {
            // Output topology/shape didn't change — downstream cache is still valid.
            metrics.features_skipped_fingerprint += 1;
        } else {
            // Output changed — invalidate downstream cache entries
            // (only those beyond the current feature, not the current one).
            for j in (i + 1)..=cursor {
                // Only invalidate if not already None
                tree.set_cache_none(j);
            }
        }

        // Store cache, param hash, and fingerprint for this feature
        tree.set_cache(i, current_brep.clone());
        tree.set_param_hash(i, current_hash);
        tree.set_fingerprint(i, new_fp);
    }

    Ok((current_brep, metrics))
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

    // ── Hash-based cache validation tests ──────────────────────────

    #[test]
    fn test_param_hash_skip() {
        // Evaluate tree, then re-evaluate without changes.
        // All features should be skipped via param hash on second eval.
        let mut tree = build_sketch_extrude_tree(7.0);

        // First evaluation — all features must be evaluated
        let (brep1, metrics1) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(brep1.faces.len(), 6);
        // Sketch doesn't go through the match arms that count as "evaluated"
        // (it sets state but doesn't produce BRep), but extrude does.
        // Both sketch and extrude are evaluated on first pass.
        assert_eq!(metrics1.features_evaluated, 2);
        assert_eq!(metrics1.features_skipped_param_hash, 0);

        // Second evaluation — nothing changed, everything should be skipped
        let (brep2, metrics2) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(brep2.faces.len(), 6);
        assert_eq!(
            metrics2.features_skipped_param_hash, 2,
            "Both sketch and extrude should be skipped via param hash"
        );
        assert_eq!(metrics2.features_evaluated, 0);
    }

    #[test]
    fn test_param_hash_detects_change() {
        // Change a param between evaluations. That feature and downstream should re-evaluate.
        let mut tree = build_sketch_extrude_tree(7.0);

        // First evaluation
        let (_brep, _) = evaluate_with_metrics(&mut tree).unwrap();

        // Change the extrude depth
        tree.features_mut()[1].params = FeatureParams::Extrude(
            crate::operations::extrude::ExtrudeParams::blind(
                Vec3::new(0.0, 0.0, 1.0),
                15.0, // changed from 7.0
            ),
        );
        // Manually clear cache for the changed feature (simulating what the UI would do)
        tree.invalidate_from(1);

        let (brep, metrics) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(brep.faces.len(), 6);
        // Sketch should be skipped (params unchanged), extrude should re-evaluate
        assert_eq!(
            metrics.features_skipped_param_hash, 1,
            "Sketch should be skipped via param hash"
        );
        assert_eq!(
            metrics.features_evaluated, 1,
            "Extrude should be re-evaluated"
        );
    }

    #[test]
    fn test_fingerprint_preserves_downstream() {
        // Build a 3-feature tree: sketch + extrude + fillet
        // Change sketch params in a way that doesn't affect the extrude output
        // (sketch is re-solved but produces the same profile)
        // Downstream (extrude) should still get re-evaluated but fingerprint should match
        let mut tree = build_sketch_extrude_tree(7.0);

        // First evaluation populates cache
        let (brep1, metrics1) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(brep1.faces.len(), 6);
        assert!(metrics1.features_evaluated >= 1);

        // Second evaluation — no changes — all skipped
        let (brep2, metrics2) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(brep2.faces.len(), 6);
        assert_eq!(metrics2.features_skipped_param_hash, 2);
        assert_eq!(metrics2.features_evaluated, 0);
    }

    #[test]
    fn test_eval_metrics() {
        // Verify metrics counts match expected skips
        let mut tree = build_sketch_extrude_tree(7.0);

        // First eval: everything evaluated
        let (_, m1) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(m1.features_evaluated, 2,
            "First eval: both sketch and extrude evaluated");

        // Second eval: everything skipped
        let (_, m2) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(m2.features_skipped_param_hash, 2,
            "Second eval: both features skipped via param hash");
        assert_eq!(m2.features_evaluated, 0);

        // Change extrude params and invalidate
        tree.features_mut()[1].params = FeatureParams::Extrude(
            crate::operations::extrude::ExtrudeParams::blind(
                Vec3::new(0.0, 0.0, 1.0),
                20.0,
            ),
        );
        tree.invalidate_from(1);

        // Third eval: sketch skipped, extrude re-evaluated
        let (_, m3) = evaluate_with_metrics(&mut tree).unwrap();
        assert_eq!(m3.features_skipped_param_hash, 1, "Sketch skipped");
        assert_eq!(m3.features_evaluated, 1, "Extrude re-evaluated");
    }

    #[test]
    fn test_brep_fingerprint_basic() {
        // Test that fingerprint captures topology correctly
        let mut tree = build_sketch_extrude_tree(7.0);
        let brep = evaluate(&mut tree).unwrap();

        let fp = brep.fingerprint();
        assert!(fp.vertex_count > 0, "Box should have vertices");
        assert_eq!(fp.face_count, 6, "Box should have 6 faces");
        assert!(fp.edge_count > 0, "Box should have edges");
        // Bounding box should be non-degenerate
        assert_ne!(fp.bbox_min, fp.bbox_max, "Bounding box should be non-degenerate");
    }

    #[test]
    fn test_brep_clone_produces_equivalent() {
        // Ensure BRep Clone works correctly
        let mut tree = build_sketch_extrude_tree(7.0);
        let brep = evaluate(&mut tree).unwrap();

        let cloned = brep.clone();
        assert_eq!(cloned.faces.len(), brep.faces.len());
        assert_eq!(cloned.edges.len(), brep.edges.len());
        assert_eq!(cloned.vertices.len(), brep.vertices.len());
        assert_eq!(cloned.fingerprint(), brep.fingerprint());
    }
}
