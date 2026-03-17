//! Assembly evaluator — evaluates each part and applies component transforms.
//! Supports recursive sub-assembly evaluation and assembly-level features.

use std::collections::HashMap;

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
/// 1. Recursively evaluate sub-assemblies (solve their internal mates first)
/// 2. Evaluate each referenced Part's FeatureTree -> base BRep
/// 3. Solve parent assembly's mate constraints (including mates referencing sub-assembly components)
/// 4. For each active Component and sub-assembly, transform BReps and return
/// 5. Apply assembly-level features (cuts/holes across components)
pub fn evaluate_assembly(assembly: &mut Assembly) -> KernelResult<AssemblyResult> {
    // Phase 1: Recursively evaluate sub-assemblies
    let mut sub_results: HashMap<String, AssemblyResult> = HashMap::new();
    for sub_ref in &mut assembly.sub_assemblies {
        if sub_ref.suppressed {
            continue;
        }
        let sub_result = evaluate_assembly(&mut sub_ref.assembly)?;
        sub_results.insert(sub_ref.component_id.clone(), sub_result);
    }

    // Phase 2: Evaluate each part (cache results by part_id)
    let mut part_breps: HashMap<String, BRep> = HashMap::new();

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

    // Phase 3: Solve parent assembly's mate constraints (updates component transforms in-place)
    if !assembly.mates.is_empty() {
        // Add sub-assembly combined BReps into the part_breps map,
        // keyed by sub-assembly component_id (used as a synthetic part_id for mates).
        for (sub_comp_id, sub_result) in &sub_results {
            if let Some(combined) = combine_breps(&sub_result.components)? {
                part_breps.insert(sub_comp_id.clone(), combined);
            }
        }
        crate::solver::assembly_solver::solve_assembly_mates(assembly, &part_breps)?;
    }

    // Phase 4: For each active component, transform its Part's BRep
    let mut results = Vec::new();
    let mut meta = Vec::new();

    for component in assembly.active_components() {
        let base_brep = part_breps.get(&component.part_id).ok_or_else(|| {
            KernelError::NotFound(format!(
                "Part BRep for '{}' not available",
                component.part_id
            ))
        })?;

        let xform = component.transform_matrix();
        let transformed_brep = transform_brep(base_brep, &xform)?;

        meta.push(ComponentMeta {
            id: component.id.clone(),
            hidden: component.hidden,
            color_override: component.color_override,
        });
        results.push((component.id.clone(), transformed_brep));
    }

    // Phase 5: Include sub-assembly components, transformed by the sub-assembly's
    // placement transform in the parent.
    for sub_ref in &assembly.sub_assemblies {
        if sub_ref.suppressed {
            continue;
        }
        if let Some(sub_result) = sub_results.get(&sub_ref.component_id) {
            let parent_xform = sub_ref.transform_matrix();
            for (comp_id, brep) in &sub_result.components {
                let transformed_brep = transform_brep(brep, &parent_xform)?;
                let sub_comp_id = format!("{}:{}", sub_ref.component_id, comp_id);
                meta.push(ComponentMeta {
                    id: sub_comp_id.clone(),
                    hidden: sub_ref.hidden,
                    color_override: None,
                });
                results.push((sub_comp_id, transformed_brep));
            }
        }
    }

    // Phase 6: Apply assembly-level features (cuts/holes across components)
    let final_results = if !assembly.assembly_features.is_empty() {
        let mut brep_map: HashMap<String, BRep> = results
            .into_iter()
            .collect();

        for feature in &assembly.assembly_features {
            crate::assembly::assembly_feature::apply_assembly_feature(&mut brep_map, feature)?;
        }

        // Rebuild results in the same order as meta
        meta.iter()
            .map(|m| {
                let brep = brep_map.remove(&m.id).unwrap();
                (m.id.clone(), brep)
            })
            .collect()
    } else {
        results
    };

    Ok(AssemblyResult { components: final_results, meta })
}

/// Transform a BRep by a 4x4 matrix, returning a new BRep.
fn transform_brep(brep: &BRep, xform: &crate::geometry::Mat4) -> KernelResult<BRep> {
    let face_polygons = extract_face_polygons(brep)?;
    let transformed_faces: Vec<(Vec<Pt3>, Vec3)> = face_polygons
        .iter()
        .map(|(points, normal)| {
            let new_points: Vec<Pt3> = points
                .iter()
                .map(|p| transform_point(xform, p))
                .collect();
            let new_normal = transform_normal(xform, normal);
            (new_points, new_normal)
        })
        .collect();
    rebuild_brep_from_faces(&transformed_faces)
}

/// Combine multiple component BReps into a single BRep (concatenating all faces).
/// Returns None if the list is empty.
fn combine_breps(components: &[(String, BRep)]) -> KernelResult<Option<BRep>> {
    if components.is_empty() {
        return Ok(None);
    }
    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();
    for (_, brep) in components {
        let face_polygons = extract_face_polygons(brep)?;
        all_faces.extend(face_polygons);
    }
    if all_faces.is_empty() {
        return Ok(None);
    }
    Ok(Some(rebuild_brep_from_faces(&all_faces)?))
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
    use crate::assembly::{Assembly, Component, Part, SubAssemblyRef};
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
    use crate::geometry::surface::plane::Plane;
    use crate::geometry::{Pt2, Vec3};
    use crate::geometry::transform;
    use crate::operations::extrude::ExtrudeParams;
    use crate::sketch::constraint::{Constraint, ConstraintKind};
    use crate::sketch::entity::SketchEntity;
    use crate::sketch::Sketch;
    use crate::topology::body::Body;
    use crate::topology::builders::extract_face_polygons;

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

        Part { id: id.into(), name: name.into(), tree, density: 1.0 }
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

        assembly.add_component(
            Component::new("comp1".into(), "part1".into(), "Origin".into())
        );
        assembly.add_component(
            Component::new("comp2".into(), "part1".into(), "Offset".into())
                .with_transform(transform::translation(20.0, 0.0, 0.0))
        );

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 2);

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

        for i in 0..3 {
            assembly.add_component(
                Component::new(format!("comp{}", i), "part1".into(), format!("Instance {}", i))
                    .with_transform(transform::translation(i as f64 * 15.0, 0.0, 0.0))
            );
        }

        let result = evaluate_assembly(&mut assembly).unwrap();
        assert_eq!(result.components.len(), 3);
    }

    // --- Sub-assembly tests ---

    fn make_sub_assembly_with_two_boxes() -> Assembly {
        let mut sub = Assembly::new();
        sub.add_part(make_box_part("sub_part1", "Sub Box A"));
        sub.add_part(make_box_part("sub_part2", "Sub Box B"));
        sub.add_component(
            Component::new("sub_comp1".into(), "sub_part1".into(), "Sub Box A".into())
        );
        sub.add_component(
            Component::new("sub_comp2".into(), "sub_part2".into(), "Sub Box B".into())
                .with_transform(transform::translation(15.0, 0.0, 0.0))
        );
        sub
    }

    #[test]
    fn sub_assembly_two_level_nesting() {
        let mut parent = Assembly::new();
        parent.add_part(make_box_part("parent_part", "Parent Box"));
        parent.add_component(
            Component::new("parent_comp".into(), "parent_part".into(), "Parent Box".into())
        );

        let sub = make_sub_assembly_with_two_boxes();
        parent.add_sub_assembly(
            SubAssemblyRef::new("sub_asm1".into(), "Sub Assembly".into(), sub)
        );

        let result = evaluate_assembly(&mut parent).unwrap();

        // 1 parent component + 2 sub-assembly components = 3
        assert_eq!(result.components.len(), 3);
        assert_eq!(result.components[0].0, "parent_comp");
        assert_eq!(result.components[1].0, "sub_asm1:sub_comp1");
        assert_eq!(result.components[2].0, "sub_asm1:sub_comp2");

        for (_, brep) in &result.components {
            assert_eq!(brep.faces.len(), 6);
            assert!(matches!(brep.body, Body::Solid(_)));
        }
    }

    #[test]
    fn sub_assembly_with_parent_transform() {
        let mut parent = Assembly::new();

        let sub = make_sub_assembly_with_two_boxes();
        parent.add_sub_assembly(
            SubAssemblyRef::new("sub1".into(), "Offset Sub".into(), sub)
                .with_transform(transform::translation(100.0, 0.0, 0.0))
        );

        let result = evaluate_assembly(&mut parent).unwrap();
        assert_eq!(result.components.len(), 2);

        let (_, brep1) = &result.components[0];
        let polys = extract_face_polygons(brep1).unwrap();
        let min_x: f64 = polys.iter()
            .flat_map(|(pts, _)| pts.iter().map(|p| p.x))
            .fold(f64::INFINITY, f64::min);
        assert!(
            min_x >= 99.9,
            "Sub-assembly component should be offset by parent transform, min_x = {}",
            min_x
        );
    }

    #[test]
    fn sub_assembly_suppressed_excluded() {
        let mut parent = Assembly::new();
        parent.add_part(make_box_part("p1", "Box"));
        parent.add_component(Component::new("c1".into(), "p1".into(), "Box".into()));

        let sub = make_sub_assembly_with_two_boxes();
        let mut sub_ref = SubAssemblyRef::new("sub1".into(), "Suppressed Sub".into(), sub);
        sub_ref.suppressed = true;
        parent.add_sub_assembly(sub_ref);

        let result = evaluate_assembly(&mut parent).unwrap();
        assert_eq!(result.components.len(), 1);
        assert_eq!(result.components[0].0, "c1");
    }
}
