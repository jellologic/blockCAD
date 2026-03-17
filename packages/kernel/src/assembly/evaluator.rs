//! Assembly evaluator — evaluates each part and applies component transforms.

use crate::error::{KernelError, KernelResult};
use crate::feature_tree::evaluator::evaluate;
use crate::geometry::{Pt3, Vec3};
use crate::geometry::transform::{transform_point, transform_normal};
use crate::topology::BRep;
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};

use super::Assembly;

/// Metadata for a component result (visibility, color).
#[derive(Debug)]
pub struct ComponentMeta {
    pub id: String,
    /// Hidden components are evaluated (for mates) but should not be rendered.
    pub hidden: bool,
    /// Per-instance color override, if any.
    pub color_override: Option<[f32; 4]>,
}

/// Result of evaluating an assembly.
#[derive(Debug)]
pub struct AssemblyResult {
    /// (component_id, transformed_brep) for each active component.
    pub components: Vec<(String, BRep)>,
    /// Metadata for each component (same order as `components`).
    pub meta: Vec<ComponentMeta>,
}

/// Evaluate an assembly, producing positioned BReps for each active component.
///
/// 1. Evaluate each referenced Part's FeatureTree → base BRep
/// 2. Solve mate constraints to compute component transforms
/// 3. For each active Component, clone the Part's BRep and apply the component transform
/// 4. Return all positioned BReps
pub fn evaluate_assembly(assembly: &mut Assembly) -> KernelResult<AssemblyResult> {
    // Step 1: Evaluate each part (cache results by part_id)
    let mut part_breps: std::collections::HashMap<String, BRep> = std::collections::HashMap::new();

    // Collect part IDs referenced by active components
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

    // Step 2: Solve mate constraints (updates component transforms in-place)
    if !assembly.mates.is_empty() {
        crate::solver::assembly_solver::solve_assembly_mates(assembly, &part_breps)?;
    }

    // Step 3: For each active component, transform its Part's BRep
    let mut results = Vec::new();
    let mut meta = Vec::new();

    for component in assembly.active_components() {
        let base_brep = part_breps.get(&component.part_id).ok_or_else(|| {
            KernelError::NotFound(format!(
                "Part BRep for '{}' not available",
                component.part_id
            ))
        })?;

        let transform = component.transform_matrix();
        let face_polygons = extract_face_polygons(base_brep)?;

        let transformed_faces: Vec<(Vec<Pt3>, Vec3)> = face_polygons
            .iter()
            .map(|(points, normal)| {
                let new_points: Vec<Pt3> = points
                    .iter()
                    .map(|p| transform_point(&transform, p))
                    .collect();
                let new_normal = transform_normal(&transform, normal);
                (new_points, new_normal)
            })
            .collect();

        let transformed_brep = rebuild_brep_from_faces(&transformed_faces)?;

        meta.push(ComponentMeta {
            id: component.id.clone(),
            hidden: component.hidden,
            color_override: component.color_override,
        });
        results.push((component.id.clone(), transformed_brep));
    }

    Ok(AssemblyResult { components: results, meta })
}

/// Evaluate an assembly with exploded view offsets applied.
///
/// Same as `evaluate_assembly`, but after positioning, each component
/// is further translated by its explosion step (if any).
pub fn evaluate_assembly_exploded(assembly: &mut Assembly) -> KernelResult<AssemblyResult> {
    let mut result = evaluate_assembly(assembly)?;

    // Apply explosion offsets
    if !assembly.explosion_steps.is_empty() {
        for (comp_id, brep) in &mut result.components {
            if let Some(step) = assembly.explosion_steps.iter().find(|s| s.component_id == *comp_id) {
                let offset = crate::geometry::transform::translation(
                    step.direction[0] * step.distance,
                    step.direction[1] * step.distance,
                    step.direction[2] * step.distance,
                );
                // Re-transform the BRep with the offset
                let face_polygons = extract_face_polygons(brep)?;
                let moved_faces: Vec<(Vec<Pt3>, Vec3)> = face_polygons
                    .iter()
                    .map(|(points, normal)| {
                        let new_points: Vec<Pt3> = points
                            .iter()
                            .map(|p| transform_point(&offset, p))
                            .collect();
                        (new_points, *normal) // translation doesn't change normals
                    })
                    .collect();
                *brep = rebuild_brep_from_faces(&moved_faces)?;
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::evaluator::evaluate;
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Mat4, Pt2, Vec3};
    use crate::geometry::transform;
    use crate::operations::extrude::ExtrudeParams;
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::Sketch;
    use crate::topology::body::Body;

    fn make_box_part(id: &str, name: &str) -> Part {
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

        Part::new(id, name, tree)
    }

    #[test]
    fn assembly_two_components_at_identity() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1", "Box A"));
        assembly.add_part(make_box_part("part2", "Box B"));
        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Box A Instance".into()));
        assembly.add_component(Component::new("comp2".into(), "part2".into(), "Box B Instance".into()));

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 2);
        for (_, brep) in &result.components {
            assert_eq!(brep.faces.len(), 6);
            assert!(matches!(brep.body, Body::Solid(_)));
        }
    }

    #[test]
    fn assembly_component_with_translation() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1", "Box"));

        // Two instances of the same part at different positions
        assembly.add_component(
            Component::new("comp1".into(), "part1".into(), "Origin".into())
        );
        assembly.add_component(
            Component::new("comp2".into(), "part1".into(), "Offset".into())
                .with_transform(transform::translation(20.0, 0.0, 0.0))
        );

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 2);

        // Both should have 6 faces
        for (_, brep) in &result.components {
            assert_eq!(brep.faces.len(), 6);
        }
    }

    #[test]
    fn assembly_suppressed_component_excluded() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1", "Box"));

        assembly.add_component(Component::new("comp1".into(), "part1".into(), "Visible".into()));
        let mut suppressed = Component::new("comp2".into(), "part1".into(), "Hidden".into());
        suppressed.suppressed = true;
        assembly.add_component(suppressed);

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].0, "comp1");
    }

    #[test]
    fn assembly_empty() {
        let mut assembly = Assembly::new();
        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 0);
    }

    #[test]
    fn assembly_same_part_multiple_instances() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_box_part("part1", "Box"));

        // 3 instances of the same part
        for i in 0..3 {
            assembly.add_component(
                Component::new(format!("comp{}", i), "part1".into(), format!("Instance {}", i))
                    .with_transform(transform::translation(i as f64 * 15.0, 0.0, 0.0))
            );
        }

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 3);
    }
}
