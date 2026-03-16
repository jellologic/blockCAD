//! GLB geometry validation tests.
//!
//! Each test builds a model via the feature tree, evaluates it, tessellates,
//! exports to GLB, writes the file to /tmp for manual inspection, then parses
//! it back with the Khronos `gltf` crate to validate structure and geometry.

use blockcad_kernel::export::gltf::{export_glb, GlbOptions};
use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Pt3, Vec3};
use blockcad_kernel::operations::chamfer::ChamferParams;
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::operations::pattern::circular::CircularPatternParams;
use blockcad_kernel::operations::pattern::linear::LinearPatternParams;
use blockcad_kernel::operations::pattern::mirror::MirrorParams;
use blockcad_kernel::operations::revolve::RevolveParams;
use blockcad_kernel::operations::shell::ShellParams;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};

// ─── Helpers ───────────────────────────────────────────────────

fn make_rectangle_sketch(width: f64, height: f64) -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(width * 0.8, 0.5) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(width * 0.8, height * 0.8) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.5, height * 0.8) });
    let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: width }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: height }, vec![p1, p2]));
    sketch
}

fn build_box_tree(width: f64, height: f64, depth: f64) -> FeatureTree {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_rectangle_sketch(width, height));
    tree.push(Feature::new(
        "e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), depth)),
    ));
    tree
}

/// Validate a GLB byte buffer with the `gltf` crate.
/// Returns (vertex_count, triangle_count, min_bounds, max_bounds).
fn validate_glb(bytes: &[u8], name: &str) -> (usize, usize, [f32; 3], [f32; 3]) {
    let (document, buffers, _) =
        gltf::import_slice(bytes).unwrap_or_else(|e| panic!("{}: gltf parse failed: {}", name, e));

    // Structure checks
    assert!(document.scenes().count() >= 1, "{}: no scenes", name);
    assert!(document.meshes().count() >= 1, "{}: no meshes", name);

    let mesh = document.meshes().next().unwrap();
    let prim = mesh.primitives().next().expect(&format!("{}: no primitives", name));
    assert_eq!(prim.mode(), gltf::mesh::Mode::Triangles, "{}: not triangle mode", name);

    // Positions
    let pos_acc = prim.get(&gltf::Semantic::Positions)
        .unwrap_or_else(|| panic!("{}: no position accessor", name));
    let vert_count = pos_acc.count();
    assert!(vert_count > 0, "{}: no vertices", name);

    // Normals
    let norm_acc = prim.get(&gltf::Semantic::Normals)
        .unwrap_or_else(|| panic!("{}: no normal accessor", name));
    assert_eq!(norm_acc.count(), vert_count, "{}: normal count mismatch", name);

    // Indices
    let idx_acc = prim.indices()
        .unwrap_or_else(|| panic!("{}: no index accessor", name));
    let idx_count = idx_acc.count();
    assert!(idx_count > 0, "{}: no indices", name);
    assert_eq!(idx_count % 3, 0, "{}: index count not multiple of 3", name);
    let tri_count = idx_count / 3;

    // Read position data and compute bounds
    let buffer_data = &buffers[0];
    let view = pos_acc.view().unwrap();
    let offset = view.offset();
    let length = view.length();
    let pos_bytes = &buffer_data[offset..offset + length];

    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for i in 0..vert_count {
        for j in 0..3 {
            let byte_off = (i * 3 + j) * 4;
            let val = f32::from_le_bytes([
                pos_bytes[byte_off],
                pos_bytes[byte_off + 1],
                pos_bytes[byte_off + 2],
                pos_bytes[byte_off + 3],
            ]);
            assert!(!val.is_nan(), "{}: NaN at vertex {} component {}", name, i, j);
            assert!(val.is_finite(), "{}: Inf at vertex {} component {}", name, i, j);
            if val < min[j] { min[j] = val; }
            if val > max[j] { max[j] = val; }
        }
    }

    (vert_count, tri_count, min, max)
}

/// Build, evaluate, tessellate, export, write to /tmp, validate, and print summary.
/// Returns (vertex_count, triangle_count, min_bounds, max_bounds, file_size).
fn run_glb_pipeline(
    tree: &mut FeatureTree,
    name: &str,
) -> (usize, usize, [f32; 3], [f32; 3], usize) {
    let brep = evaluate(tree)
        .unwrap_or_else(|e| panic!("{}: evaluate failed: {}", name, e));
    let mesh = tessellate_brep(&brep, &TessellationParams::default())
        .unwrap_or_else(|e| panic!("{}: tessellate failed: {}", name, e));

    assert!(mesh.triangle_count() > 0, "{}: zero triangles from tessellation", name);
    mesh.validate().unwrap_or_else(|e| panic!("{}: mesh validation failed: {}", name, e));

    let glb_bytes = export_glb(&mesh, name, &GlbOptions::default())
        .unwrap_or_else(|e| panic!("{}: export_glb failed: {}", name, e));
    assert!(glb_bytes.len() > 12, "{}: GLB too small", name);

    // Write to /tmp for manual inspection in Blender
    let path = format!("/tmp/blockcad_test_{}.glb", name);
    std::fs::write(&path, &glb_bytes)
        .unwrap_or_else(|e| panic!("{}: failed to write {}: {}", name, path, e));

    let file_size = glb_bytes.len();

    // Parse back and validate
    let (verts, tris, min, max) = validate_glb(&glb_bytes, name);

    eprintln!(
        "{}: {} verts, {} tris, {} bytes  bounds=[{:.2},{:.2},{:.2}]..[{:.2},{:.2},{:.2}]  -> {}",
        name, verts, tris, file_size,
        min[0], min[1], min[2], max[0], max[1], max[2], path,
    );

    (verts, tris, min, max, file_size)
}

// ─── Test 1: Simple box ────────────────────────────────────────

#[test]
fn glb_simple_box() {
    let mut tree = build_box_tree(20.0, 10.0, 5.0);
    let (verts, tris, min, max, _) = run_glb_pipeline(&mut tree, "simple_box");

    // A box should have 6 faces, 12 triangles, 24 vertices (unshared normals)
    assert_eq!(tris, 12, "simple_box: expected 12 triangles, got {}", tris);
    assert_eq!(verts, 24, "simple_box: expected 24 vertices, got {}", verts);

    // Bounds should be ~0..20, ~0..10, ~0..5
    let tol = 0.5;
    assert!(max[0] - min[0] > 20.0 - tol, "simple_box: X extent too small");
    assert!(max[1] - min[1] > 10.0 - tol, "simple_box: Y extent too small");
    assert!(max[2] - min[2] > 5.0 - tol, "simple_box: Z extent too small");
}

// ─── Test 2: Tall thin tower ───────────────────────────────────

#[test]
fn glb_tall_thin_tower() {
    let mut tree = build_box_tree(2.0, 2.0, 50.0);
    let (verts, tris, min, max, _) = run_glb_pipeline(&mut tree, "tall_thin_tower");

    assert!(tris >= 12, "tall_thin_tower: expected >= 12 triangles");
    assert!(verts >= 24, "tall_thin_tower: expected >= 24 vertices");

    // Z dimension should be ~50
    let z_extent = max[2] - min[2];
    assert!(
        (z_extent - 50.0).abs() < 1.0,
        "tall_thin_tower: Z extent = {:.2}, expected ~50.0", z_extent,
    );
}

// ─── Test 3: Symmetric extrude ─────────────────────────────────

#[test]
fn glb_symmetric_extrude() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_rectangle_sketch(10.0, 5.0));

    let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 8.0);
    params.symmetric = true;
    tree.push(Feature::new(
        "e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(params),
    ));

    let (verts, tris, min, max, _) = run_glb_pipeline(&mut tree, "symmetric_extrude");

    assert!(tris >= 12, "symmetric_extrude: expected >= 12 triangles");
    assert!(verts >= 24, "symmetric_extrude: expected >= 24 vertices");

    // Symmetric: should extend from -4 to +4 in Z
    let z_mid = (max[2] + min[2]) / 2.0;
    assert!(
        z_mid.abs() < 1.0,
        "symmetric_extrude: Z center = {:.2}, expected ~0.0", z_mid,
    );
    let z_extent = max[2] - min[2];
    assert!(
        (z_extent - 8.0).abs() < 1.0,
        "symmetric_extrude: Z extent = {:.2}, expected ~8.0", z_extent,
    );
}

// ─── Test 4: Extrude with draft ────────────────────────────────

#[test]
fn glb_extrude_with_draft() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_rectangle_sketch(10.0, 10.0));

    let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0);
    params.draft_angle = 0.1; // radians (~5.7 degrees)
    tree.push(Feature::new(
        "e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(params),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "extrude_with_draft");

    // Draft creates tapered sides; still 6 faces but geometry differs
    assert!(tris >= 12, "extrude_with_draft: expected >= 12 triangles");
    assert!(verts >= 24, "extrude_with_draft: expected >= 24 vertices");
}

// ─── Test 5: Box with fillet ───────────────────────────────────

#[test]
fn glb_box_with_fillet() {
    let mut tree = build_box_tree(10.0, 5.0, 7.0);
    tree.push(Feature::new(
        "f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "box_with_fillet");

    // Fillet adds curved segments: more faces than a plain box
    assert!(tris > 12, "box_with_fillet: expected > 12 triangles, got {}", tris);
    assert!(verts > 24, "box_with_fillet: expected > 24 vertices, got {}", verts);
}

// ─── Test 6: Box with chamfer ──────────────────────────────────

#[test]
fn glb_box_with_chamfer() {
    let mut tree = build_box_tree(10.0, 5.0, 7.0);
    tree.push(Feature::new(
        "ch1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams {
            edge_indices: vec![0],
            distance: 1.0,
            distance2: None,
        }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "box_with_chamfer");

    // Chamfer adds one flat face: 7 faces -> 14 triangles, 28 vertices
    assert!(tris > 12, "box_with_chamfer: expected > 12 triangles, got {}", tris);
    assert!(verts > 24, "box_with_chamfer: expected > 24 vertices, got {}", verts);
}

// ─── Test 7: Box with shell ────────────────────────────────────

#[test]
fn glb_box_with_shell() {
    let mut tree = build_box_tree(10.0, 5.0, 7.0);
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1],
            thickness: 0.5,
        }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "box_with_shell");

    // Shell: 5 outer + 5 inner + 4 rim = 14 faces -> 28 triangles
    assert!(tris > 12, "box_with_shell: expected > 12 triangles, got {}", tris);
    assert!(verts > 24, "box_with_shell: expected > 24 vertices, got {}", verts);
}

// ─── Test 8: Box with cut extrude ──────────────────────────────

#[test]
fn glb_box_with_cut() {
    let mut tree = build_box_tree(10.0, 5.0, 7.0);

    // Add a second sketch for the cut (smaller rect inside)
    let mut cut_sketch = Sketch::new(Plane::xy(0.0));
    let p0 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 1.0) });
    let p1 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 1.0) });
    let p2 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 4.0) });
    let p3 = cut_sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 4.0) });
    cut_sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    cut_sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    cut_sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    cut_sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 4.0 }, vec![p0, p1]));
    cut_sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 3.0 }, vec![p1, p2]));

    tree.push(Feature::new(
        "s2".into(), "Cut Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder,
    ));
    tree.sketches.insert(2, cut_sketch);
    tree.push(Feature::new(
        "ce1".into(), "Cut Extrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0)),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "box_with_cut");

    // Cut adds internal faces
    assert!(tris > 12, "box_with_cut: expected > 12 triangles, got {}", tris);
    assert!(verts > 24, "box_with_cut: expected > 24 vertices, got {}", verts);
}

// ─── Test 9: Linear pattern ───────────────────────────────────

#[test]
fn glb_linear_pattern() {
    let mut tree = build_box_tree(10.0, 5.0, 5.0);
    tree.push(Feature::new(
        "lp1".into(), "Linear Pattern".into(), FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 3,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "linear_pattern");

    // 3 copies * 6 faces * 2 tris = 36 triangles minimum
    assert!(tris >= 36, "linear_pattern: expected >= 36 triangles, got {}", tris);
    assert!(verts >= 72, "linear_pattern: expected >= 72 vertices, got {}", verts);
}

// ─── Test 10: Circular pattern ─────────────────────────────────

#[test]
fn glb_circular_pattern() {
    let mut tree = build_box_tree(10.0, 5.0, 5.0);
    tree.push(Feature::new(
        "cp1".into(), "Circular Pattern".into(), FeatureKind::CircularPattern,
        FeatureParams::CircularPattern(CircularPatternParams {
            axis_origin: Pt3::new(0.0, 0.0, 0.0),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            count: 4,
            total_angle: 2.0 * std::f64::consts::PI,
        }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "circular_pattern");

    // 4 copies * 6 faces * 2 tris = 48 triangles minimum
    assert!(tris >= 48, "circular_pattern: expected >= 48 triangles, got {}", tris);
    assert!(verts >= 96, "circular_pattern: expected >= 96 vertices, got {}", verts);
}

// ─── Test 11: Mirror ───────────────────────────────────────────

#[test]
fn glb_mirror() {
    let mut tree = build_box_tree(10.0, 5.0, 5.0);
    tree.push(Feature::new(
        "m1".into(), "Mirror".into(), FeatureKind::Mirror,
        FeatureParams::Mirror(MirrorParams {
            plane_origin: Pt3::new(15.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        }),
    ));

    let (verts, tris, _min, _max, _) = run_glb_pipeline(&mut tree, "mirror");

    // 2 copies * 6 faces * 2 tris = 24 triangles minimum
    assert!(tris >= 24, "mirror: expected >= 24 triangles, got {}", tris);
    assert!(verts >= 48, "mirror: expected >= 48 vertices, got {}", verts);
}

// ─── Test 12: Full revolve ─────────────────────────────────────

#[test]
fn glb_revolve_full() {
    let mut tree = FeatureTree::new();

    // Create a sketch for revolve: small rectangle offset from Z axis
    // Plane is XZ (normal = -Y) so sketch u=X, v=Z
    let mut sketch = Sketch::new(Plane {
        origin: Pt3::new(2.0, 0.0, 0.0),
        normal: Vec3::new(0.0, -1.0, 0.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    });
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(2.0, 2.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 2.0) });
    sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 2.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 2.0 }, vec![p1, p2]));

    tree.push(Feature::new(
        "s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, sketch);
    tree.push(Feature::new(
        "r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams::full(
            Pt3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        )),
    ));

    let (verts, tris, min, max, _) = run_glb_pipeline(&mut tree, "revolve_full");

    // Full revolution of a rectangle creates a toroidal shape with many faces
    assert!(tris > 12, "revolve_full: expected many triangles, got {}", tris);
    assert!(verts > 24, "revolve_full: expected many vertices, got {}", verts);

    // Revolution around Z: X and Y extents should be symmetric
    let x_extent = max[0] - min[0];
    let y_extent = max[1] - min[1];
    assert!(
        (x_extent - y_extent).abs() < 1.0,
        "revolve_full: X extent ({:.2}) and Y extent ({:.2}) should be similar", x_extent, y_extent,
    );
    // Outer radius is 4 (2 offset + 2 width), so diameter ~ 8
    assert!(
        x_extent > 6.0,
        "revolve_full: X extent = {:.2}, expected > 6.0 for revolution", x_extent,
    );
}
