use crate::error::{KernelError, KernelResult};
use crate::operations::extrude::extrude_profile;
use crate::operations::revolve::revolve_profile;
use crate::sketch::profile::extract_profile;
use crate::sketch::solver_bridge::build_constraint_graph;
use crate::solver::newton_raphson::{solve, SolverConfig};
use crate::topology::BRep;

use super::feature::FeatureState;
use super::kind::FeatureKind;
use super::params::FeatureParams;
use super::tree::FeatureTree;

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
                let params = match &tree.features()[i].params {
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
                current_brep = extrude_profile(&profile, params.direction, params.depth)?;
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
                current_brep = revolve_profile(
                    &profile,
                    params.axis_origin,
                    params.axis_direction,
                    params.angle,
                )?;
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
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams {
                direction: Vec3::new(0.0, 0.0, 1.0),
                depth,
                symmetric: false,
                draft_angle: 0.0,
            }),
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
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams {
                direction: Vec3::new(0.0, 0.0, 1.0),
                depth: 5.0,
                symmetric: false,
                draft_angle: 0.0,
            }),
        ));

        let result = evaluate(&mut tree);
        assert!(result.is_err(), "Extrude without sketch should fail");
    }
}
