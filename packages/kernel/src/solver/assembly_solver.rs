//! Assembly solver — builds a constraint graph from assembly mates
//! and solves for component transforms using the existing Newton-Raphson solver.

use std::collections::HashMap;

use crate::assembly::{Assembly, GeometryRef, MateKind};
use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

use super::equations_3d::*;
use super::graph::ConstraintGraph;
use super::newton_raphson::{solve, SolverConfig, SolverResult};
use super::variable::Variable;
use crate::geometry::transform;

/// Solve assembly mates, updating component transforms.
///
/// The first component is grounded (fixed). Remaining components are free (6 DOF each).
/// Mates generate equations that constrain relative positions.
///
/// `part_breps` maps part_id → evaluated BRep (needed to extract face geometry).
pub fn solve_assembly_mates(
    assembly: &mut Assembly,
    part_breps: &HashMap<String, BRep>,
) -> KernelResult<SolverResult> {
    if assembly.components.is_empty() {
        return Ok(SolverResult { converged: true, iterations: 0, residual: 0.0 });
    }

    let active_mates: Vec<_> = assembly.mates.iter().filter(|m| !m.suppressed).cloned().collect();
    if active_mates.is_empty() {
        return Ok(SolverResult { converged: true, iterations: 0, residual: 0.0 });
    }

    let mut graph = ConstraintGraph::new();

    // Create variables: grounded components are fixed, others are free.
    // If no component is explicitly grounded, the first active component is grounded.
    let has_explicit_ground = assembly.components.iter().any(|c| !c.suppressed && c.grounded);
    let mut first_active_seen = false;
    let mut comp_vars: HashMap<String, ComponentVars> = HashMap::new();
    for comp in assembly.components.iter() {
        if comp.suppressed { continue; }

        let current = comp.transform_matrix();
        let t = transform::get_translation(&current);

        let is_grounded = comp.grounded || (!has_explicit_ground && !first_active_seen);
        first_active_seen = true;

        if is_grounded {
            let vars = ComponentVars {
                tx: graph.variables.add(Variable::fixed(t.x)),
                ty: graph.variables.add(Variable::fixed(t.y)),
                tz: graph.variables.add(Variable::fixed(t.z)),
                rx: graph.variables.add(Variable::fixed(0.0)),
                ry: graph.variables.add(Variable::fixed(0.0)),
                rz: graph.variables.add(Variable::fixed(0.0)),
            };
            comp_vars.insert(comp.id.clone(), vars);
        } else {
            let vars = ComponentVars {
                tx: graph.variables.add(Variable::new(t.x)),
                ty: graph.variables.add(Variable::new(t.y)),
                tz: graph.variables.add(Variable::new(t.z)),
                rx: graph.variables.add(Variable::new(0.0)),
                ry: graph.variables.add(Variable::new(0.0)),
                rz: graph.variables.add(Variable::new(0.0)),
            };
            comp_vars.insert(comp.id.clone(), vars);
        }
    }

    // Build equations from mates
    for mate in &active_mates {
        let vars_a = comp_vars.get(&mate.component_a).ok_or_else(|| {
            KernelError::NotFound(format!("Component '{}' not found for mate", mate.component_a))
        })?;
        let vars_b = comp_vars.get(&mate.component_b).ok_or_else(|| {
            KernelError::NotFound(format!("Component '{}' not found for mate", mate.component_b))
        })?;

        // Get the part BReps for geometry extraction
        let comp_a = assembly.components.iter().find(|c| c.id == mate.component_a)
            .ok_or_else(|| KernelError::NotFound(format!("Component '{}'", mate.component_a)))?;
        let comp_b = assembly.components.iter().find(|c| c.id == mate.component_b)
            .ok_or_else(|| KernelError::NotFound(format!("Component '{}'", mate.component_b)))?;

        let brep_a = part_breps.get(&comp_a.part_id).ok_or_else(|| {
            KernelError::NotFound(format!("BRep for part '{}'", comp_a.part_id))
        })?;
        let brep_b = part_breps.get(&comp_b.part_id).ok_or_else(|| {
            KernelError::NotFound(format!("BRep for part '{}'", comp_b.part_id))
        })?;

        match &mate.kind {
            MateKind::Coincident => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;

                // Coincident: face distance = 0 (coplanar constraint)
                graph.add_equation(Box::new(CoincidentDistanceEquation::new(
                    *vars_a, *vars_b, face_a, face_b,
                )));
            }
            MateKind::Concentric => {
                let axis_a = extract_axis_geometry(brep_a, &mate.geometry_ref_a)?;
                let axis_b = extract_axis_geometry(brep_b, &mate.geometry_ref_b)?;

                graph.add_equation(Box::new(ConcentricDistanceEquation::new(
                    *vars_a, *vars_b, axis_a.clone(), axis_b.clone(),
                )));
                graph.add_equation(Box::new(ConcentricAlignEquation::new(
                    *vars_a, *vars_b, axis_a, axis_b,
                )));
            }
            MateKind::Distance { value } => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;

                graph.add_equation(Box::new(DistanceMateEquation::new(
                    *vars_a, *vars_b, face_a, face_b, *value,
                )));
            }
            MateKind::Angle { value } => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;

                graph.add_equation(Box::new(AngleMateEquation::new(
                    *vars_a, *vars_b, face_a, face_b, *value,
                )));
            }
            MateKind::Parallel => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(ParallelEquation::new(*vars_a, *vars_b, face_a, face_b)));
            }
            MateKind::Perpendicular => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(PerpendicularEquation::new(*vars_a, *vars_b, face_a, face_b)));
            }
            MateKind::Tangent => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(TangentEquation::new(*vars_a, *vars_b, face_a, face_b)));
            }
            MateKind::Lock => {
                let cv = vars_b;
                graph.add_equation(Box::new(LockEquation::new(cv.tx, graph.variables.value(cv.tx))));
                graph.add_equation(Box::new(LockEquation::new(cv.ty, graph.variables.value(cv.ty))));
                graph.add_equation(Box::new(LockEquation::new(cv.tz, graph.variables.value(cv.tz))));
                graph.add_equation(Box::new(LockEquation::new(cv.rx, graph.variables.value(cv.rx))));
                graph.add_equation(Box::new(LockEquation::new(cv.ry, graph.variables.value(cv.ry))));
                graph.add_equation(Box::new(LockEquation::new(cv.rz, graph.variables.value(cv.rz))));
            }
            MateKind::Hinge => {
                // Hinge = same as concentric (axis collinear, rotation allowed)
                let axis_a = extract_axis_geometry(brep_a, &mate.geometry_ref_a)?;
                let axis_b = extract_axis_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(ConcentricDistanceEquation::new(
                    *vars_a, *vars_b, axis_a.clone(), axis_b.clone(),
                )));
                graph.add_equation(Box::new(ConcentricAlignEquation::new(
                    *vars_a, *vars_b, axis_a, axis_b,
                )));
            }
            MateKind::Gear { ratio } => {
                // Couple rotation of component_a and component_b
                graph.add_equation(Box::new(GearEquation::new(vars_a.rx, vars_b.rx, *ratio)));
            }
            MateKind::Screw { pitch } => {
                // Couple translation Z with rotation X of component_b
                graph.add_equation(Box::new(ScrewEquation::new(vars_b.tz, vars_b.rx, *pitch)));
            }
            MateKind::Limit { min, max } => {
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(LimitEquation::new(
                    *vars_a, *vars_b, face_a, face_b, *min, *max,
                )));
            }
            MateKind::Width => {
                // Width: center component_b between face_a and face_b of component_a
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(WidthEquation::new(
                    *vars_a, *vars_b, face_a, face_b,
                )));
            }
            MateKind::Symmetric => {
                // Symmetric: point_a and point_b equidistant from midplane
                let face_a = extract_face_geometry(brep_a, &mate.geometry_ref_a)?;
                let face_b = extract_face_geometry(brep_b, &mate.geometry_ref_b)?;
                graph.add_equation(Box::new(SymmetricEquation::new(
                    *vars_a, *vars_b, face_a, face_b,
                )));
            }
            MateKind::RackPinion { pitch_radius } => {
                // Couple rack translation X with pinion rotation X
                graph.add_equation(Box::new(RackPinionEquation::new(vars_a.tx, vars_b.rx, *pitch_radius)));
            }
            MateKind::Cam { eccentricity, base_radius } => {
                // Cam: follower ty (comp_b) = base_radius + eccentricity * cos(rx of comp_a)
                graph.add_equation(Box::new(CamEquation::new(
                    vars_b.ty, vars_a.rx, *eccentricity, *base_radius,
                )));
            }
            MateKind::Slot { axis } => {
                // Slot: constrain comp_b to slide along the given axis through comp_a
                let slot_axis = Vec3::new(axis[0], axis[1], axis[2]);
                let (eq_u, eq_v) = SlotEquation::pair(*vars_a, *vars_b, slot_axis);
                graph.add_equation(Box::new(eq_u));
                graph.add_equation(Box::new(eq_v));
            }
            MateKind::UniversalJoint => {
                // Universal joint (Hooke's joint): axes intersect at a point,
                // each component retains rotational freedom about its own axis.
                let axis_a = extract_axis_geometry(brep_a, &mate.geometry_ref_a)?;
                let axis_b = extract_axis_geometry(brep_b, &mate.geometry_ref_b)?;
                // 3 equations for point coincidence (x, y, z)
                graph.add_equation(Box::new(UniversalJointPointXEquation::new(
                    *vars_a, *vars_b, axis_a.clone(), axis_b.clone(),
                )));
                graph.add_equation(Box::new(UniversalJointPointYEquation::new(
                    *vars_a, *vars_b, axis_a.clone(), axis_b.clone(),
                )));
                graph.add_equation(Box::new(UniversalJointPointZEquation::new(
                    *vars_a, *vars_b, axis_a.clone(), axis_b.clone(),
                )));
                // 1 equation for no-twist (perpendicular axes)
                graph.add_equation(Box::new(UniversalJointNoTwistEquation::new(
                    *vars_a, *vars_b, axis_a, axis_b,
                )));
            }
        }
    }

    // Solve
    let config = SolverConfig {
        max_iterations: 200,
        tolerance: 1e-8,
    };
    let result = solve(&mut graph, &config)?;

    // Write solved transforms back to components
    if result.converged {
        for comp in &mut assembly.components {
            if comp.suppressed { continue; }
            if let Some(vars) = comp_vars.get(&comp.id) {
                let new_transform = vars.build_transform(&graph.variables);
                comp.transform = transform::to_array(&new_transform);
            }
        }
    }

    Ok(result)
}

/// Extract face geometry (centroid + normal) from a BRep face by index.
fn extract_face_geometry(brep: &BRep, geom_ref: &GeometryRef) -> KernelResult<FaceGeometry> {
    let face_idx = match geom_ref {
        GeometryRef::Face(i) => *i,
        _ => return Err(KernelError::InvalidParameter {
            param: "geometry_ref".into(),
            value: "Expected Face reference for this mate type".into(),
        }),
    };

    let (_, face) = brep.faces.iter().nth(face_idx).ok_or_else(|| {
        KernelError::NotFound(format!("Face index {}", face_idx))
    })?;

    let surf_idx = face.surface_index.ok_or_else(|| {
        KernelError::Topology("Face has no surface".into())
    })?;
    let normal = brep.surfaces[surf_idx].normal_at(0.0, 0.0)?;

    // Compute face centroid from loop vertices
    let loop_id = face.outer_loop.ok_or_else(|| {
        KernelError::Topology("Face has no outer loop".into())
    })?;
    let loop_ = brep.loops.get(loop_id)?;

    let mut sum = Vec3::new(0.0, 0.0, 0.0);
    let mut count = 0;
    for &coedge_id in &loop_.coedges {
        let coedge = brep.coedges.get(coedge_id)?;
        let edge = brep.edges.get(coedge.edge)?;
        let start_vid = match coedge.orientation {
            crate::topology::edge::Orientation::Forward => edge.start,
            crate::topology::edge::Orientation::Reversed => edge.end,
        };
        let vertex = brep.vertices.get(start_vid)?;
        sum += Vec3::new(vertex.point.x, vertex.point.y, vertex.point.z);
        count += 1;
    }

    let centroid = if count > 0 {
        Pt3::new(sum.x / count as f64, sum.y / count as f64, sum.z / count as f64)
    } else {
        Pt3::origin()
    };

    Ok(FaceGeometry { point: centroid, normal })
}

/// Extract axis geometry from a BRep face (for cylindrical faces, uses the surface normal as axis direction).
/// For flat faces, uses the face normal as the axis direction and centroid as a point on the axis.
fn extract_axis_geometry(brep: &BRep, geom_ref: &GeometryRef) -> KernelResult<AxisGeometry> {
    let face_geom = extract_face_geometry(brep, geom_ref)?;
    Ok(AxisGeometry {
        point: face_geom.point,
        direction: face_geom.normal.normalize(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, GeometryRef, Mate, MateKind, Part};
    use crate::solver::equation::Equation;
    use crate::solver::equations_3d::ComponentVars;
    use crate::feature_tree::evaluator::evaluate;
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
        sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 10.0 }, vec![p0, p1]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 5.0 }, vec![p1, p2]));

        let mut tree = FeatureTree::new();
        tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
        tree.sketches.insert(0, sketch);
        tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
            FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));

        Part { id: id.into(), name: format!("Box {}", id), tree, density: 1.0 }
    }

    fn evaluate_parts(assembly: &mut Assembly) -> HashMap<String, BRep> {
        let mut breps = HashMap::new();
        for part in &mut assembly.parts {
            let brep = evaluate(&mut part.tree).unwrap();
            breps.insert(part.id.clone(), brep);
        }
        breps
    }

    #[test]
    fn solve_coincident_mate_moves_component() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1"));
        assembly.add_part(make_box_part("part2"));

        // Component A at origin (will be grounded)
        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Box A".into()));
        // Component B offset slightly — solver should move it to satisfy mate
        assembly.add_component(
            Component::new("comp2".into(), "part2".into(), "Box B".into())
                .with_transform(transform::translation(0.0, 0.0, 10.0))
        );

        // Coincident mate: top face of A (face 1) with bottom face of B (face 0)
        assembly.mates.push(Mate {
            id: "mate1".into(),
            kind: MateKind::Coincident,
            component_a: "comp1".into(),
            component_b: "comp2".into(),
            geometry_ref_a: GeometryRef::Face(1), // top face of A
            geometry_ref_b: GeometryRef::Face(0), // bottom face of B
            suppressed: false,
        });

        let part_breps = evaluate_parts(&mut assembly);
        let result = solve_assembly_mates(&mut assembly, &part_breps).unwrap();
        assert!(result.converged, "Assembly solver should converge");

        // Verify: the face-to-face distance should be ~0 after solving
        // (exact tz depends on solver path with underdetermined system,
        //  but the constraint residual should be near zero)
        assert!(result.residual < 0.01,
            "Coincident mate residual should be near zero, got {:.6}", result.residual);
    }

    #[test]
    fn solve_distance_mate() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1"));
        assembly.add_part(make_box_part("part2"));

        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Box A".into()));
        assembly.add_component(
            Component::new("comp2".into(), "part2".into(), "Box B".into())
                .with_transform(transform::translation(0.0, 0.0, 15.0))
        );

        // Distance mate: 5mm gap between top of A and bottom of B
        assembly.mates.push(Mate {
            id: "mate1".into(),
            kind: MateKind::Distance { value: 5.0 },
            component_a: "comp1".into(),
            component_b: "comp2".into(),
            geometry_ref_a: GeometryRef::Face(1),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        let part_breps = evaluate_parts(&mut assembly);
        let result = solve_assembly_mates(&mut assembly, &part_breps).unwrap();
        assert!(result.converged, "Distance mate solver should converge");

        // Verify: the constraint residual should be near zero
        assert!(result.residual < 0.01,
            "Distance mate residual should be near zero, got {:.6}", result.residual);
    }

    #[test]
    fn rack_pinion_solver_converges() {
        use crate::solver::variable::Variable;
        use crate::solver::equations_3d::RackPinionEquation;
        use super::super::graph::ConstraintGraph;
        use super::super::newton_raphson::{solve, SolverConfig};

        let mut graph = ConstraintGraph::new();
        let pitch_radius = 5.0;
        let target_rotation = std::f64::consts::PI;

        // Rack: free tx (starts at 0, solver must find correct value)
        let rack_tx = graph.variables.add(Variable::new(0.0));

        // Pinion: rx is fixed at PI
        let pinion_rx = graph.variables.add(Variable::fixed(target_rotation));

        // Rack-pinion coupling: tx_rack = rx_pinion * pitch_radius
        graph.add_equation(Box::new(RackPinionEquation::new(rack_tx, pinion_rx, pitch_radius)));

        let config = SolverConfig { max_iterations: 50, tolerance: 1e-10 };
        let result = solve(&mut graph, &config).unwrap();
        assert!(result.converged, "Rack-pinion solver should converge");

        let solved_tx = graph.variables.value(rack_tx);
        let expected = target_rotation * pitch_radius;
        assert!((solved_tx - expected).abs() < 1e-6,
            "Rack tx should be PI * 5.0 = {:.6}, got {:.6}", expected, solved_tx);
    }

    #[test]
    fn solve_slot_mate_x_axis() {
        // Slot mate along X axis: comp_b starts at (3, 5, 7).
        // The solver should zero out Y and Z while X remains free.
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1"));
        assembly.add_part(make_box_part("part2"));

        assembly.add_component(
            Component::new("comp1".into(), "part1".into(), "Box A".into())
                .with_grounded(true)
        );
        assembly.add_component(
            Component::new("comp2".into(), "part2".into(), "Box B".into())
                .with_transform(transform::translation(3.0, 5.0, 7.0))
        );

        assembly.mates.push(Mate {
            id: "slot1".into(),
            kind: MateKind::Slot { axis: [1.0, 0.0, 0.0] },
            component_a: "comp1".into(),
            component_b: "comp2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });

        let part_breps = evaluate_parts(&mut assembly);
        let result = solve_assembly_mates(&mut assembly, &part_breps).unwrap();
        assert!(result.converged, "Slot mate solver should converge, residual={:.6}", result.residual);
        assert!(result.residual < 0.01, "Slot mate residual should be near zero, got {:.6}", result.residual);

        // After solving, comp_b's Y and Z should be ~0, X can be anything
        let comp_b = assembly.components.iter().find(|c| c.id == "comp2").unwrap();
        let t = comp_b.transform_matrix();
        let pos = transform::get_translation(&t);
        assert!(pos.y.abs() < 0.1, "Y should be ~0 after slot solve, got {:.4}", pos.y);
        assert!(pos.z.abs() < 0.1, "Z should be ~0 after slot solve, got {:.4}", pos.z);
    }

    #[test]
    fn no_mates_is_noop() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1"));
        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Box".into()));

        let part_breps = evaluate_parts(&mut assembly);
        let result = solve_assembly_mates(&mut assembly, &part_breps).unwrap();
        assert!(result.converged);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn solve_universal_joint_mate() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1"));
        assembly.add_part(make_box_part("part2"));

        // Component A at origin (will be grounded)
        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Box A".into()));
        // Component B offset — solver should move it so axes intersect
        assembly.add_component(
            Component::new("comp2".into(), "part2".into(), "Box B".into())
                .with_transform(transform::translation(2.0, 3.0, 4.0))
        );

        // Universal joint mate using face references.
        // Face 0 has Z-normal, face 2 has a side normal (X or Y) — perpendicular axes.
        assembly.mates.push(Mate {
            id: "mate1".into(),
            kind: MateKind::UniversalJoint,
            component_a: "comp1".into(),
            component_b: "comp2".into(),
            geometry_ref_a: GeometryRef::Face(0), // bottom face, Z-normal axis
            geometry_ref_b: GeometryRef::Face(2), // side face, perpendicular axis
            suppressed: false,
        });

        let part_breps = evaluate_parts(&mut assembly);
        let result = solve_assembly_mates(&mut assembly, &part_breps).unwrap();
        assert!(result.converged, "Universal joint solver should converge, residual={:.6}", result.residual);
        assert!(result.residual < 0.01,
            "Universal joint residual should be near zero, got {:.6}", result.residual);
    }

    #[test]
    fn universal_joint_point_coincidence_after_solve() {
        // Unit test directly on the equation structs to verify point coincidence
        use crate::solver::variable::Variable;

        let mut vars = super::super::variable::VariableStore::new();

        // comp_a grounded at origin, axis along Z through origin
        let comp_a = ComponentVars {
            tx: vars.add(Variable::fixed(0.0)),
            ty: vars.add(Variable::fixed(0.0)),
            tz: vars.add(Variable::fixed(0.0)),
            rx: vars.add(Variable::fixed(0.0)),
            ry: vars.add(Variable::fixed(0.0)),
            rz: vars.add(Variable::fixed(0.0)),
        };

        // comp_b at origin, axis along X through origin — already satisfies constraints
        let comp_b = ComponentVars {
            tx: vars.add(Variable::new(0.0)),
            ty: vars.add(Variable::new(0.0)),
            tz: vars.add(Variable::new(0.0)),
            rx: vars.add(Variable::new(0.0)),
            ry: vars.add(Variable::new(0.0)),
            rz: vars.add(Variable::new(0.0)),
        };

        let axis_a = AxisGeometry { point: Pt3::origin(), direction: Vec3::new(0.0, 0.0, 1.0) };
        let axis_b = AxisGeometry { point: Pt3::origin(), direction: Vec3::new(1.0, 0.0, 0.0) };

        // All 4 equations should be satisfied (axes intersect at origin, are perpendicular)
        let eq_x = UniversalJointPointXEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());
        let eq_y = UniversalJointPointYEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());
        let eq_z = UniversalJointPointZEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());
        let eq_twist = UniversalJointNoTwistEquation::new(comp_a, comp_b, axis_a, axis_b);

        assert!(eq_x.eval(&vars).abs() < 1e-9, "X point coincidence");
        assert!(eq_y.eval(&vars).abs() < 1e-9, "Y point coincidence");
        assert!(eq_z.eval(&vars).abs() < 1e-9, "Z point coincidence");
        assert!(eq_twist.eval(&vars).abs() < 1e-9, "No-twist (perpendicular axes)");
    }

    #[test]
    fn universal_joint_retains_rotational_freedom() {
        // Verify that each component can rotate about its own axis
        // without violating the universal joint constraints.
        use crate::solver::variable::Variable;

        let mut vars = super::super::variable::VariableStore::new();

        // comp_a grounded at origin, axis along Z
        let comp_a = ComponentVars {
            tx: vars.add(Variable::fixed(0.0)),
            ty: vars.add(Variable::fixed(0.0)),
            tz: vars.add(Variable::fixed(0.0)),
            rx: vars.add(Variable::fixed(0.0)),
            ry: vars.add(Variable::fixed(0.0)),
            rz: vars.add(Variable::fixed(0.0)),
        };

        // comp_b at origin, axis along X, rotated 45 degrees about X axis
        let comp_b = ComponentVars {
            tx: vars.add(Variable::new(0.0)),
            ty: vars.add(Variable::new(0.0)),
            tz: vars.add(Variable::new(0.0)),
            rx: vars.add(Variable::new(0.7)),  // rotation about X
            ry: vars.add(Variable::new(0.0)),
            rz: vars.add(Variable::new(0.0)),
        };

        let axis_a = AxisGeometry { point: Pt3::origin(), direction: Vec3::new(0.0, 0.0, 1.0) };
        let axis_b = AxisGeometry { point: Pt3::origin(), direction: Vec3::new(1.0, 0.0, 0.0) };

        // Point coincidence should still hold (rotation about own axis at origin stays at origin)
        let eq_x = UniversalJointPointXEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());
        let eq_y = UniversalJointPointYEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());
        let eq_z = UniversalJointPointZEquation::new(comp_a, comp_b, axis_a.clone(), axis_b.clone());

        assert!(eq_x.eval(&vars).abs() < 1e-9, "X should be zero after rotation about own axis");
        assert!(eq_y.eval(&vars).abs() < 1e-9, "Y should be zero after rotation about own axis");
        assert!(eq_z.eval(&vars).abs() < 1e-9, "Z should be zero after rotation about own axis");

        // The no-twist constraint may or may not be satisfied depending on rotation,
        // since rotating comp_b about X axis changes the world-space direction of the X axis.
        // This is expected — the DOF is "rotation about own axis" which the solver permits
        // by not over-constraining (only 4 equations for 6 DOF = 2 free DOF).
    }
}
