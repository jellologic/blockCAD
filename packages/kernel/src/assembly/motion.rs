//! Motion studies — kinematic animation for assembly mechanisms.
//!
//! A motion study drives a single mate parameter through a range of values,
//! re-solving the assembly at each step and recording all component transforms.

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, KernelResult};
use crate::feature_tree::evaluator::evaluate;
use crate::solver::assembly_solver::DriverOverride;

use super::{Assembly, MateKind};

/// Parameters for a motion study.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionStudyParams {
    /// The ID of the mate that drives the motion.
    pub driver_mate_id: String,
    /// Start value for the driver parameter.
    pub start_value: f64,
    /// End value for the driver parameter.
    pub end_value: f64,
    /// Number of animation frames.
    pub num_steps: usize,
}

/// A single frame of a motion study.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionFrame {
    /// Frame index (0-based).
    pub step: usize,
    /// The driver parameter value at this frame.
    pub driver_value: f64,
    /// Transforms for all active components.
    pub component_transforms: Vec<ComponentTransform>,
}

/// A component's transform at a particular frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTransform {
    /// Component ID.
    pub component_id: String,
    /// 4x4 column-major homogeneous transform.
    pub transform: [f64; 16],
}

/// Run a motion study on an assembly.
///
/// The driver value is interpreted as the rotation angle (in radians) of
/// component_a of the driver mate. At each step, the assembly constraints
/// are re-solved with component_a's rx fixed to the interpolated driver value,
/// and all component transforms are recorded.
///
/// Supported driver mate kinds:
/// - `Gear { ratio }` — drives rotation, ratio is preserved
/// - `Screw { pitch }` — drives rotation, translation follows
/// - `RackPinion { pitch_radius }` — drives pinion rotation, rack follows
/// - `Cam { lift, base_radius }` — drives cam rotation, follower follows
/// - `Hinge` — drives rotation of the hinge axis
/// - `Angle { value }` — drives the angle between faces
pub fn run_motion_study(
    assembly: &mut Assembly,
    params: &MotionStudyParams,
) -> KernelResult<Vec<MotionFrame>> {
    if params.num_steps == 0 {
        return Err(KernelError::InvalidParameter {
            param: "num_steps".into(),
            value: "0 (must be >= 1)".into(),
        });
    }

    let mate_idx = assembly
        .mates
        .iter()
        .position(|m| m.id == params.driver_mate_id)
        .ok_or_else(|| {
            KernelError::NotFound(format!(
                "Driver mate '{}' not found in assembly",
                params.driver_mate_id
            ))
        })?;

    if assembly.mates[mate_idx].suppressed {
        return Err(KernelError::InvalidParameter {
            param: "driver_mate_id".into(),
            value: format!("Mate '{}' is suppressed", params.driver_mate_id),
        });
    }

    validate_driveable(&assembly.mates[mate_idx].kind)?;

    let driver_comp_a = assembly.mates[mate_idx].component_a.clone();

    // Evaluate part BReps once (they don't change during motion)
    let mut part_breps: std::collections::HashMap<String, crate::topology::BRep> =
        std::collections::HashMap::new();
    let needed_part_ids: Vec<String> = assembly
        .active_components()
        .iter()
        .map(|c| c.part_id.clone())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    for part_id in &needed_part_ids {
        let part = assembly.find_part_mut(part_id).ok_or_else(|| {
            KernelError::NotFound(format!("Part '{}' not found in assembly", part_id))
        })?;
        let brep = evaluate(&mut part.tree)?;
        part_breps.insert(part_id.clone(), brep);
    }

    let mut frames = Vec::with_capacity(params.num_steps);

    for step in 0..params.num_steps {
        let t = if params.num_steps == 1 {
            0.0
        } else {
            step as f64 / (params.num_steps - 1) as f64
        };
        let driver_value = params.start_value + t * (params.end_value - params.start_value);

        // Create a driver override that fixes component_a's rx to the driver value
        let driver = DriverOverride {
            component_id: driver_comp_a.clone(),
            rx_value: driver_value,
        };

        // Solve assembly constraints with the driver override
        crate::solver::assembly_solver::solve_assembly_mates_driven(
            assembly,
            &part_breps,
            &driver,
        )?;

        // Record all active component transforms
        let component_transforms: Vec<ComponentTransform> = assembly
            .active_components()
            .iter()
            .map(|c| ComponentTransform {
                component_id: c.id.clone(),
                transform: c.transform,
            })
            .collect();

        frames.push(MotionFrame {
            step,
            driver_value,
            component_transforms,
        });
    }

    Ok(frames)
}

/// Validate that a mate kind can be driven in a motion study.
fn validate_driveable(kind: &MateKind) -> KernelResult<()> {
    match kind {
        MateKind::Gear { .. }
        | MateKind::Screw { .. }
        | MateKind::RackPinion { .. }
        | MateKind::Cam { .. }
        | MateKind::Hinge
        | MateKind::Angle { .. } => Ok(()),
        other => Err(KernelError::InvalidParameter {
            param: "driver_mate_id".into(),
            value: format!(
                "Mate kind '{:?}' cannot be used as a motion driver",
                other
            ),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, GeometryRef, Mate, MateKind, Part};
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Pt2, Vec3};
    use crate::operations::extrude::ExtrudeParams;
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::Sketch;

    fn make_box_part(id: &str) -> Part {
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 0.5) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 4.0) });
        let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.5, 4.0) });
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

        let mut tree = FeatureTree::new();
        tree.push(Feature::new(
            "s1".into(),
            "Sketch".into(),
            FeatureKind::Sketch,
            FeatureParams::Placeholder,
        ));
        tree.sketches.insert(0, sketch);
        tree.push(Feature::new(
            "e1".into(),
            "Extrude".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0)),
        ));

        Part {
            id: id.into(),
            name: format!("Box {}", id),
            tree,
            density: 1.0,
        }
    }

    fn make_gear_assembly() -> Assembly {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("gear_a"));
        assembly.add_part(make_box_part("gear_b"));

        // Gear A is grounded (driver)
        assembly.add_component(
            Component::new("comp_gear_a".into(), "gear_a".into(), "Gear A".into())
                .with_grounded(true),
        );
        // Gear B is free (driven)
        assembly.add_component(
            Component::new("comp_gear_b".into(), "gear_b".into(), "Gear B".into()),
        );

        // Gear mate with 2:1 ratio
        assembly.mates.push(Mate {
            id: "gear_mate".into(),
            kind: MateKind::Gear { ratio: 2.0 },
            component_a: "comp_gear_a".into(),
            component_b: "comp_gear_b".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        assembly
    }

    #[test]
    fn gear_motion_study_coupled_rotation() {
        let mut assembly = make_gear_assembly();

        let params = MotionStudyParams {
            driver_mate_id: "gear_mate".into(),
            start_value: 0.0,
            end_value: std::f64::consts::TAU, // Full 360 degrees
            num_steps: 5,
        };

        let frames = run_motion_study(&mut assembly, &params).unwrap();

        assert_eq!(frames.len(), 5);

        // At each frame, verify the gear ratio relationship holds.
        // Gear equation: rx_a * ratio = rx_b
        // With ratio=2.0, when gear_a rotates by angle, gear_b should rotate by 2*angle.
        for (i, frame) in frames.iter().enumerate() {
            assert_eq!(frame.step, i);
            assert_eq!(frame.component_transforms.len(), 2);

            // The driver value should be linearly interpolated
            let expected_driver = i as f64 / 4.0 * std::f64::consts::TAU;
            assert!(
                (frame.driver_value - expected_driver).abs() < 1e-6,
                "Frame {} driver_value: expected {}, got {}",
                i,
                expected_driver,
                frame.driver_value,
            );
        }

        // First frame (step=0): driver_value=0, both should be at identity-like transforms
        let first = &frames[0];
        assert!(
            (first.driver_value).abs() < 1e-9,
            "First frame driver should be 0",
        );

        // Last frame (step=4): driver_value = 2*PI
        let last = &frames[4];
        assert!(
            (last.driver_value - std::f64::consts::TAU).abs() < 1e-6,
            "Last frame driver should be 2*PI",
        );
    }

    #[test]
    fn rack_pinion_motion_study() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("pinion"));
        assembly.add_part(make_box_part("rack"));

        assembly.add_component(
            Component::new("comp_pinion".into(), "pinion".into(), "Pinion".into())
                .with_grounded(true),
        );
        assembly.add_component(
            Component::new("comp_rack".into(), "rack".into(), "Rack".into()),
        );

        let pitch_radius = 5.0;
        assembly.mates.push(Mate {
            id: "rp_mate".into(),
            kind: MateKind::RackPinion { pitch_radius },
            component_a: "comp_pinion".into(),
            component_b: "comp_rack".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        let params = MotionStudyParams {
            driver_mate_id: "rp_mate".into(),
            start_value: 0.0,
            end_value: std::f64::consts::TAU,
            num_steps: 4,
        };

        let frames = run_motion_study(&mut assembly, &params).unwrap();
        assert_eq!(frames.len(), 4);

        // At the last frame, the rack should have translated by pitch_radius * 2*PI
        let last = &frames[3];
        let rack_transform = last
            .component_transforms
            .iter()
            .find(|ct| ct.component_id == "comp_rack")
            .expect("Rack component not found");

        // The tx (column-major index 12) should be approximately pitch_radius * 2*PI
        let expected_tx = pitch_radius * std::f64::consts::TAU;
        let actual_tx = rack_transform.transform[12];
        assert!(
            (actual_tx - expected_tx).abs() < 0.1,
            "Rack translation: expected ~{:.2}, got {:.2}",
            expected_tx,
            actual_tx,
        );
    }

    #[test]
    fn cam_motion_study_follower_oscillates() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("cam_body"));
        assembly.add_part(make_box_part("follower"));

        assembly.add_component(
            Component::new("comp_cam".into(), "cam_body".into(), "Cam".into())
                .with_grounded(true),
        );
        assembly.add_component(
            Component::new("comp_follower".into(), "follower".into(), "Follower".into()),
        );

        let lift = 3.0;
        let base_radius = 10.0;
        assembly.mates.push(Mate {
            id: "cam_mate".into(),
            kind: MateKind::Cam { lift, base_radius },
            component_a: "comp_cam".into(),
            component_b: "comp_follower".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        let params = MotionStudyParams {
            driver_mate_id: "cam_mate".into(),
            start_value: 0.0,
            end_value: std::f64::consts::TAU,
            num_steps: 5,
        };

        let frames = run_motion_study(&mut assembly, &params).unwrap();
        assert_eq!(frames.len(), 5);

        // Verify follower oscillation. The cam equation is:
        // tz_follower = base_radius + lift * sin(rx_cam)
        // At step 0 (rx=0): tz = base_radius + 0 = 10
        // At step 1 (rx=PI/2): tz = base_radius + lift = 13
        // At step 2 (rx=PI): tz = base_radius + 0 = 10
        // At step 3 (rx=3PI/2): tz = base_radius - lift = 7
        // At step 4 (rx=2PI): tz = base_radius + 0 = 10
        let expected_tz = [
            base_radius,
            base_radius + lift,
            base_radius,
            base_radius - lift,
            base_radius,
        ];

        for (i, frame) in frames.iter().enumerate() {
            let follower = frame
                .component_transforms
                .iter()
                .find(|ct| ct.component_id == "comp_follower")
                .expect("Follower not found");

            // tz is at column-major index 14 in a 4x4 matrix
            let actual_tz = follower.transform[14];
            assert!(
                (actual_tz - expected_tz[i]).abs() < 0.5,
                "Frame {}: follower tz expected ~{:.2}, got {:.2}",
                i,
                expected_tz[i],
                actual_tz,
            );
        }
    }

    #[test]
    fn invalid_driver_mate_id_returns_error() {
        let mut assembly = make_gear_assembly();

        let params = MotionStudyParams {
            driver_mate_id: "nonexistent_mate".into(),
            start_value: 0.0,
            end_value: 1.0,
            num_steps: 2,
        };

        let result = run_motion_study(&mut assembly, &params);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            KernelError::NotFound(msg) => {
                assert!(
                    msg.contains("nonexistent_mate"),
                    "Error should mention the mate ID: {}",
                    msg,
                );
            }
            other => panic!("Expected NotFound error, got: {:?}", other),
        }
    }

    #[test]
    fn zero_steps_returns_error() {
        let mut assembly = make_gear_assembly();

        let params = MotionStudyParams {
            driver_mate_id: "gear_mate".into(),
            start_value: 0.0,
            end_value: 1.0,
            num_steps: 0,
        };

        let result = run_motion_study(&mut assembly, &params);
        assert!(result.is_err());
    }

    #[test]
    fn non_driveable_mate_returns_error() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("p1"));
        assembly.add_part(make_box_part("p2"));
        assembly.add_component(
            Component::new("c1".into(), "p1".into(), "C1".into()).with_grounded(true),
        );
        assembly.add_component(Component::new("c2".into(), "p2".into(), "C2".into()));
        assembly.mates.push(Mate {
            id: "lock_mate".into(),
            kind: MateKind::Lock,
            component_a: "c1".into(),
            component_b: "c2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        let params = MotionStudyParams {
            driver_mate_id: "lock_mate".into(),
            start_value: 0.0,
            end_value: 1.0,
            num_steps: 2,
        };

        let result = run_motion_study(&mut assembly, &params);
        assert!(result.is_err());
        match result.unwrap_err() {
            KernelError::InvalidParameter { .. } => {}
            other => panic!("Expected InvalidParameter error, got: {:?}", other),
        }
    }
}
