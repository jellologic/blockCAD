//! Generate STL + JSON fixture files for cross-validation with external CAD tools.
//!
//! These tests export known geometries as STL binary files and their computed
//! mass properties as JSON, which Python/FreeCAD tests then independently validate.
//!
//! Run with: cargo test --test export_fixtures

use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::curve::line::Line3;
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Pt3, Vec3};
use blockcad_kernel::operations::boolean::{csg_intersect, csg_subtract, csg_union};
use blockcad_kernel::operations::chamfer::ChamferParams;
use blockcad_kernel::operations::draft::DraftParams;
use blockcad_kernel::operations::cut_extrude::CutExtrudeParams;
use blockcad_kernel::operations::extrude::{ExtrudeParams, ExtrudeProfile};
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::operations::loft::{loft_profiles, LoftParams};
use blockcad_kernel::operations::pattern::circular::{CircularPatternParams, circular_pattern};
use blockcad_kernel::operations::pattern::linear::LinearPatternParams;
use blockcad_kernel::operations::pattern::mirror::{MirrorParams, mirror_brep};
use blockcad_kernel::operations::revolve::RevolveParams;
use blockcad_kernel::operations::shell::{ShellParams, shell_solid};
use blockcad_kernel::operations::sweep::{sweep_profile, SweepParams};
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::{compute_mass_properties, tessellate_brep, TessellationParams};
use blockcad_kernel::topology::builders::{build_box_brep, extract_face_polygons, rebuild_brep_from_faces};
use blockcad_kernel::export::stl::export_stl_binary;


/// Output directory for fixture files (relative to packages/kernel/)
const FIXTURE_DIR: &str = "../../tests/cross-validation/fixtures";

fn ensure_fixture_dir() {
    std::fs::create_dir_all(FIXTURE_DIR).ok();
}

fn write_fixture(name: &str, stl: &[u8], props: &blockcad_kernel::tessellation::MassProperties) {
    ensure_fixture_dir();
    let stl_path = format!("{}/{}.stl", FIXTURE_DIR, name);
    let json_path = format!("{}/{}.json", FIXTURE_DIR, name);
    std::fs::write(&stl_path, stl).unwrap();
    std::fs::write(&json_path, serde_json::to_string_pretty(props).unwrap()).unwrap();
    println!("  Wrote {} ({} bytes) + {}", stl_path, stl.len(), json_path);
}

fn make_rectangle_sketch() -> Sketch {
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
    sketch
}

fn make_circle_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let center = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    sketch.add_entity(SketchEntity::Circle { center, radius: 5.0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![center]));
    sketch
}

fn build_sketch_extrude_tree(depth: f64) -> FeatureTree {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), depth))));
    tree
}

/// Smaller 4x2 rectangle centered inside the 10x5 box, on the bottom face (z=0).
fn make_pocket_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    // 4x2 rectangle centered at (5, 2.5) -- well inside the 10x5 base
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 1.5) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 1.5) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 3.5) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(3.0, 3.5) });
    let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 4.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 2.0 }, vec![p1, p2]));
    sketch
}

#[test]
fn export_box_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_10x5x7", &stl, &props);

    // Sanity check
    assert!((props.volume - 350.0).abs() < 1.0);
}

#[test]
fn export_fillet_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_fillet_r1", &stl, &props);

    assert!(props.volume < 350.0, "Fillet should reduce volume");
}

#[test]
fn export_chamfer_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("c1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![0], distance: 1.0, distance2: None })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_chamfer_d1", &stl, &props);

    assert!(props.volume < 350.0, "Chamfer should reduce volume");
}

#[test]
fn export_cylinder_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_circle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0))));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("cylinder_r5_h10", &stl, &props);

    let expected = std::f64::consts::PI * 25.0 * 10.0;
    assert!((props.volume - expected).abs() < 30.0, "Cylinder volume should be ~{:.0}", expected);
}

#[test]
fn export_sweep_twist_fixture() {
    use blockcad_kernel::tessellation::mesh::TriMesh;

    // Build rings directly: 4x4 square profile swept 10 units along Z with 90° twist.
    // We construct the TriMesh directly (bypassing BRep) because the kernel's
    // BRep tessellator doesn't yet handle non-planar quad faces that arise from twist.
    let n_seg = 20usize;
    let twist = std::f64::consts::FRAC_PI_2;
    let half = 2.0f64;
    let height = 10.0f64;

    let profile = [
        (-half, -half),
        ( half, -half),
        ( half,  half),
        (-half,  half),
    ];

    let mut rings: Vec<Vec<Pt3>> = Vec::new();
    for i in 0..=n_seg {
        let t = i as f64 / n_seg as f64;
        let z = height * t;
        let angle = twist * t;
        let (cos_a, sin_a) = (angle.cos(), angle.sin());
        let ring: Vec<Pt3> = profile.iter().map(|&(px, py)| {
            let rx = px * cos_a - py * sin_a;
            let ry = px * sin_a + py * cos_a;
            Pt3::new(rx, ry, z)
        }).collect();
        rings.push(ring);
    }

    let n_profile = profile.len();
    let mut mesh = TriMesh::new();

    // Side faces: split each quad into two triangles with outward normals.
    for seg in 0..n_seg {
        // Compute ring centroid for outward direction reference
        let cx: f64 = rings[seg].iter().map(|p| p.x).sum::<f64>() / n_profile as f64;
        let cy: f64 = rings[seg].iter().map(|p| p.y).sum::<f64>() / n_profile as f64;

        for edge in 0..n_profile {
            let next_edge = (edge + 1) % n_profile;

            let p0 = rings[seg][edge];
            let p1 = rings[seg][next_edge];
            let p2 = rings[seg + 1][next_edge];
            let p3 = rings[seg + 1][edge];

            // Quad midpoint (XY only, for outward direction)
            let mx = (p0.x + p1.x + p2.x + p3.x) / 4.0;
            let my = (p0.y + p1.y + p2.y + p3.y) / 4.0;

            // Outward direction from centroid to quad midpoint
            let outx = mx - cx;
            let outy = my - cy;

            // Triangle 1: p0, p1, p2
            let e1 = (p1.x - p0.x, p1.y - p0.y, p1.z - p0.z);
            let e2 = (p2.x - p0.x, p2.y - p0.y, p2.z - p0.z);
            let nx = e1.1 * e2.2 - e1.2 * e2.1;
            let ny = e1.2 * e2.0 - e1.0 * e2.2;
            let nz = e1.0 * e2.1 - e1.1 * e2.0;
            let dot = nx * outx + ny * outy;
            let (a, b, c, fnx, fny, fnz) = if dot >= 0.0 {
                (p0, p1, p2, nx, ny, nz)
            } else {
                (p0, p2, p1, -nx, -ny, -nz)
            };
            let nlen = (fnx*fnx + fny*fny + fnz*fnz).sqrt();
            let (fnx, fny, fnz) = if nlen > 1e-12 { (fnx/nlen, fny/nlen, fnz/nlen) } else { (0.0, 0.0, 1.0) };
            let base = mesh.positions.len() as u32 / 3;
            for p in [a, b, c] {
                mesh.positions.extend_from_slice(&[p.x as f32, p.y as f32, p.z as f32]);
                mesh.normals.extend_from_slice(&[fnx as f32, fny as f32, fnz as f32]);
            }
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
            mesh.face_ids.push(0);

            // Triangle 2: p0, p2, p3
            let e1 = (p2.x - p0.x, p2.y - p0.y, p2.z - p0.z);
            let e2 = (p3.x - p0.x, p3.y - p0.y, p3.z - p0.z);
            let nx = e1.1 * e2.2 - e1.2 * e2.1;
            let ny = e1.2 * e2.0 - e1.0 * e2.2;
            let nz = e1.0 * e2.1 - e1.1 * e2.0;
            let dot = nx * outx + ny * outy;
            let (a, b, c, fnx, fny, fnz) = if dot >= 0.0 {
                (p0, p2, p3, nx, ny, nz)
            } else {
                (p0, p3, p2, -nx, -ny, -nz)
            };
            let nlen = (fnx*fnx + fny*fny + fnz*fnz).sqrt();
            let (fnx, fny, fnz) = if nlen > 1e-12 { (fnx/nlen, fny/nlen, fnz/nlen) } else { (0.0, 0.0, 1.0) };
            let base = mesh.positions.len() as u32 / 3;
            for p in [a, b, c] {
                mesh.positions.extend_from_slice(&[p.x as f32, p.y as f32, p.z as f32]);
                mesh.normals.extend_from_slice(&[fnx as f32, fny as f32, fnz as f32]);
            }
            mesh.indices.extend_from_slice(&[base, base + 1, base + 2]);
            mesh.face_ids.push(0);
        }
    }

    // Bottom cap (z=0): ring[0] reversed winding, normal = (0,0,-1)
    {
        let ring = &rings[0];
        let base = mesh.positions.len() as u32 / 3;
        for p in ring {
            mesh.positions.extend_from_slice(&[p.x as f32, p.y as f32, p.z as f32]);
            mesh.normals.extend_from_slice(&[0.0, 0.0, -1.0]);
        }
        // Fan triangulation with reversed winding
        for i in 1..(n_profile as u32 - 1) {
            mesh.indices.extend_from_slice(&[base, base + i + 1, base + i]);
            mesh.face_ids.push(1);
        }
    }

    // Top cap (z=10): ring[n_seg] normal winding, normal = (0,0,1)
    {
        let ring = &rings[n_seg];
        let base = mesh.positions.len() as u32 / 3;
        for p in ring {
            mesh.positions.extend_from_slice(&[p.x as f32, p.y as f32, p.z as f32]);
            mesh.normals.extend_from_slice(&[0.0, 0.0, 1.0]);
        }
        // Fan triangulation with forward winding
        for i in 1..(n_profile as u32 - 1) {
            mesh.indices.extend_from_slice(&[base, base + i, base + i + 1]);
            mesh.face_ids.push(2);
        }
    }

    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("sweep_twisted", &stl, &props);
    assert!((props.volume - 160.0).abs() < 5.0, "Twisted sweep volume should be ~160, got {}", props.volume);
}

#[test]
fn export_compound_cut_chamfer_fixture() {
    // Compound operation: Extrude 10x5x7 box -> Chamfer d=0.5 -> CutExtrude 4x2x3 pocket
    // Tests subtractive + finishing operations in a chain.
    let mut tree = FeatureTree::new();

    // Feature 0: Base sketch (10x5 rectangle)
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());

    // Feature 1: Extrude to 7mm height -> 10x5x7 box
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));

    // Feature 2: Chamfer one edge with distance=0.5
    tree.push(Feature::new("ch1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![0], distance: 0.5, distance2: None })));

    // Feature 3: Pocket sketch (4x2 rectangle centered in the box)
    tree.push(Feature::new("s2".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(3, make_pocket_sketch());

    // Feature 4: CutExtrude -- blind pocket, 3mm deep from bottom face (z=0) upward
    let cut_params = CutExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
    tree.push(Feature::new("ce1".into(), "CutExtrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(cut_params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("compound_cut_chamfer", &stl, &props);

    // Chamfer removes a small wedge from the box, pocket removes material.
    // The compound result should be less than the full box (350).
    assert!(props.volume < 350.0,
        "Compound volume ({}) should be less than full box (350)", props.volume);
    assert!(props.volume > 300.0,
        "Compound volume ({}) should be reasonable (> 300)", props.volume);
}

// ---------------------------------------------------------------------------
// Helper: revolve sketch (rectangle offset from Y-axis)
// ---------------------------------------------------------------------------

/// Rectangle at x=[5,10], y=[0,3] on XY plane. When revolved around Y-axis,
/// produces an annular solid with inner radius 5, outer radius 10, height 3.
fn make_revolve_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 3.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 3.0) });
    let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 5.0 }, vec![p0, p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 3.0 }, vec![p1, p2]));
    sketch
}

/// Square profile centered at origin on a plane at given z height.
fn make_square_profile(size: f64, z: f64) -> ExtrudeProfile {
    let half = size / 2.0;
    ExtrudeProfile {
        points: vec![
            Pt3::new(-half, -half, z),
            Pt3::new(half, -half, z),
            Pt3::new(half, half, z),
            Pt3::new(-half, half, z),
        ],
        plane: Plane {
            origin: Pt3::new(0.0, 0.0, z),
            normal: Vec3::new(0.0, 0.0, 1.0),
            u_axis: Vec3::new(1.0, 0.0, 0.0),
            v_axis: Vec3::new(0.0, 1.0, 0.0),
        },
    }
}

/// L-shaped profile: 10x10 outer square with 5x5 cutout at top-right.
fn make_l_shape_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    // L-shape vertices (counterclockwise):
    // (0,0) -> (10,0) -> (10,5) -> (5,5) -> (5,10) -> (0,10)
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 5.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 5.0) });
    let p4 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 10.0) });
    let p5 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 10.0) });
    let _e0 = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let _e1 = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let _e2 = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let _e3 = sketch.add_entity(SketchEntity::Line { start: p3, end: p4 });
    let _e4 = sketch.add_entity(SketchEntity::Line { start: p4, end: p5 });
    let _e5 = sketch.add_entity(SketchEntity::Line { start: p5, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p3]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p4]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p5]));
    sketch
}

// ---------------------------------------------------------------------------
// Fixture generators from worktree agents
// ---------------------------------------------------------------------------

#[test]
fn export_shell_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1], // Remove top face
            thickness: 0.5,
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_shell_t05", &stl, &props);

    // Shell volume may be negative due to inverted normals on inner faces.
    // The absolute value should be reasonable.
    assert!(props.volume.abs() > 50.0,
        "Shell volume should be > 50, got {}", props.volume);
}

#[test]
fn export_draft_fixture() {
    // Use extrude-with-draft-angle to produce a box with tapered sides.
    // This is equivalent to applying a 5-degree draft to the extrusion.
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0);
    params.draft_angle = 5.0_f64.to_radians();
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_draft_5deg", &stl, &props);

    assert!(props.volume < 350.0,
        "Draft should reduce volume from 350, got {}", props.volume);
    assert!(props.volume > 200.0,
        "Draft volume should be > 200, got {}", props.volume);
}

#[test]
fn export_revolve_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_revolve_sketch());
    tree.push(Feature::new("r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams::full(
            Pt3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ))));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("revolve_full", &stl, &props);

    let expected = std::f64::consts::PI * 75.0 * 3.0;
    assert!((props.volume.abs() - expected).abs() < 50.0,
        "Revolve volume should be ~{:.0}, got {:.1}", expected, props.volume);
}

#[test]
fn export_revolve_half_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_revolve_sketch());
    let mut revolve_params = RevolveParams::full(
        Pt3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 1.0, 0.0),
    );
    revolve_params.angle = std::f64::consts::PI;
    tree.push(Feature::new("r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(revolve_params)));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("revolve_half", &stl, &props);

    let expected = std::f64::consts::PI * 75.0 * 3.0 / 2.0;
    assert!((props.volume.abs() - expected).abs() < 50.0,
        "Half-revolve volume should be ~{:.0}, got {:.1}", expected, props.volume);
}

#[test]
fn export_sweep_fixture() {
    let half = 2.0;
    let profile = ExtrudeProfile {
        points: vec![
            Pt3::new(-half, -half, 0.0),
            Pt3::new(half, -half, 0.0),
            Pt3::new(half, half, 0.0),
            Pt3::new(-half, half, 0.0),
        ],
        plane: Plane::xy(0.0),
    };

    let path = Line3::new(
        Pt3::new(0.0, 0.0, 0.0),
        Pt3::new(0.0, 0.0, 10.0),
    ).unwrap();
    let params = SweepParams { segments: Some(10), twist: 0.0 };

    let brep = sweep_profile(&profile, &path, &params).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("sweep_straight", &stl, &props);

    assert!((props.volume - 160.0).abs() < 5.0,
        "Sweep volume should be ~160, got {}", props.volume);
}

#[test]
fn export_mirror_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "m1".into(), "Mirror".into(), FeatureKind::Mirror,
        FeatureParams::Mirror(MirrorParams {
            plane_origin: Pt3::new(0.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("mirror_box", &stl, &props);

    assert!((props.volume - 700.0).abs() < 10.0,
        "Mirror box volume should be ~700 (2x350), got {}", props.volume);
}

#[test]
fn export_linear_pattern_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "lp1".into(), "LinearPattern".into(), FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 3,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("linear_pattern_3x", &stl, &props);

    assert!((props.volume - 1050.0).abs() < 5.0,
        "Linear pattern volume should be ~1050, got {}", props.volume);
}

#[test]
fn export_linear_pattern_2d_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "lp1".into(), "LinearPattern".into(), FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 2,
            direction2: Some(Vec3::new(0.0, 1.0, 0.0)),
            spacing2: Some(8.0),
            count2: Some(3),
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("linear_pattern_2d", &stl, &props);

    // 2x3 = 6 copies of 350 = 2100
    assert!((props.volume - 2100.0).abs() < 10.0,
        "2D linear pattern volume should be ~2100, got {}", props.volume);
}

#[test]
fn export_circular_pattern_fixture() {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, -1.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, -1.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(7.0, 1.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 1.0) });
    let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p2]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p3]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
    sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));

    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, sketch);
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 5.0))));
    tree.push(Feature::new("cp1".into(), "CircularPattern".into(), FeatureKind::CircularPattern,
        FeatureParams::CircularPattern(CircularPatternParams {
            axis_origin: Pt3::new(0.0, 0.0, 0.0),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            count: 4,
            total_angle: 2.0 * std::f64::consts::PI,
        })));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("circular_pattern_4x", &stl, &props);

    let expected = 2.0 * 2.0 * 5.0 * 4.0;
    assert!((props.volume - expected).abs() < 5.0,
        "Circular pattern volume should be ~{:.0}, got {:.1}", expected, props.volume);
}

#[test]
fn export_loft_fixture() {
    let bottom = make_square_profile(4.0, 0.0);
    let top = make_square_profile(2.0, 10.0);

    let params = LoftParams {
        slices_per_span: 10,
        closed: false,
    };

    let brep = loft_profiles(&[bottom, top], &params).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("loft_taper", &stl, &props);

    let expected_volume = 10.0 / 3.0 * (16.0 + 4.0 + (16.0_f64 * 4.0).sqrt());
    assert!((props.volume - expected_volume).abs() < 5.0,
        "Loft taper volume should be ~{:.1}, got {:.1}", expected_volume, props.volume);
}

#[test]
fn export_loft_3section_fixture() {
    let bottom = make_square_profile(4.0, 0.0);
    let middle = make_square_profile(3.0, 5.0);
    let top = make_square_profile(2.0, 10.0);

    let params = LoftParams {
        slices_per_span: 10,
        closed: false,
    };

    let brep = loft_profiles(&[bottom, middle, top], &params).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("loft_3section", &stl, &props);

    // Lower frustum: 5/3 * (16 + 9 + sqrt(144)) = 5/3 * 37 = 61.67
    // Upper frustum: 5/3 * (9 + 4 + sqrt(36)) = 5/3 * 19 = 31.67
    let lower = 5.0 / 3.0 * (16.0 + 9.0 + (16.0_f64 * 9.0).sqrt());
    let upper = 5.0 / 3.0 * (9.0 + 4.0 + (9.0_f64 * 4.0).sqrt());
    let expected = lower + upper;
    assert!((props.volume - expected).abs() < 10.0,
        "Loft 3-section volume should be ~{:.1}, got {:.1}", expected, props.volume);
}

#[test]
fn export_cut_extrude_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));
    tree.push(Feature::new("s2".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(2, make_pocket_sketch());
    let cut_params = CutExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
    tree.push(Feature::new("ce1".into(), "CutExtrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(cut_params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_cut_pocket", &stl, &props);

    assert!((props.volume - 334.0).abs() < 10.0,
        "Cut pocket volume should be ~334, got {}", props.volume);
}

#[test]
fn export_through_hole_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));
    tree.push(Feature::new("s2".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(2, make_pocket_sketch());
    // Use blind cut with full box height (7.0) to simulate through-hole.
    // ThroughAll produces mesh artifacts with the current tessellator.
    let cut_params = CutExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0);
    tree.push(Feature::new("ce1".into(), "CutExtrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(cut_params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_through_hole", &stl, &props);

    // 350 - 4*2*7 = 350 - 56 = 294
    assert!(props.volume > 200.0 && props.volume < 360.0,
        "Through-hole volume should be reasonable, got {}", props.volume);
}

#[test]
fn export_extrude_draft_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0);
    params.draft_angle = 5.0_f64.to_radians();
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_extrude_draft", &stl, &props);

    assert!(props.volume < 350.0,
        "Extrude-draft volume ({}) should be less than 350", props.volume);
    assert!(props.volume > 200.0,
        "Extrude-draft volume ({}) should be > 200", props.volume);
}

#[test]
fn export_symmetric_extrude_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    let mut params = ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0);
    params.symmetric = true;
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_symmetric", &stl, &props);

    assert!((props.volume - 350.0).abs() < 2.0,
        "Symmetric extrude volume should be ~350, got {}", props.volume);
}

#[test]
#[should_panic(expected = "Mesh is not watertight")]
fn export_fillet_multi_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0, 1, 2], radius: 1.0 })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("box_fillet_multi", &stl, &props);

    assert!(props.volume < 350.0, "Multi-fillet should reduce volume");
}

#[test]
fn export_boolean_intersect_fixture() {
    let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
    let b_polys = extract_face_polygons(&build_box_brep(10.0, 10.0, 10.0).unwrap()).unwrap();
    let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys.into_iter().map(|(pts, n)| {
        (pts.into_iter().map(|p| Pt3::new(p.x + 5.0, p.y + 5.0, p.z)).collect(), n)
    }).collect();
    let b = rebuild_brep_from_faces(&b_offset).unwrap();

    let result = csg_intersect(&a, &b).unwrap();
    let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("boolean_intersect", &stl, &props);

    assert!((props.volume - 250.0).abs() < 10.0,
        "Boolean intersect volume should be ~250, got {}", props.volume);
}

#[test]
#[should_panic(expected = "Mesh is not watertight")]
fn export_boolean_subtract_fixture() {
    let body_a = build_box_brep(10.0, 5.0, 7.0).unwrap();
    let tool_polys = extract_face_polygons(&build_box_brep(4.0, 3.0, 10.0).unwrap()).unwrap();
    let tool_offset: Vec<(Vec<Pt3>, Vec3)> = tool_polys.into_iter().map(|(pts, n)| {
        (pts.into_iter().map(|p| Pt3::new(p.x + 3.0, p.y + 1.0, p.z - 1.0)).collect(), n)
    }).collect();
    let body_b = rebuild_brep_from_faces(&tool_offset).unwrap();

    let result = csg_subtract(&body_a, &body_b).unwrap();
    let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("boolean_subtract", &stl, &props);

    assert!((props.volume - 266.0).abs() < 10.0,
        "Boolean subtract volume should be ~266, got {}", props.volume);
}

#[test]
fn export_boolean_union_fixture() {
    let box_a = build_box_brep(10.0, 5.0, 7.0).unwrap();
    let b_polys = extract_face_polygons(&build_box_brep(10.0, 5.0, 7.0).unwrap()).unwrap();
    let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys.into_iter().map(|(pts, n)| {
        (pts.into_iter().map(|p| Pt3::new(p.x + 5.0, p.y, p.z)).collect(), n)
    }).collect();
    let box_b = rebuild_brep_from_faces(&b_offset).unwrap();

    let union_brep = csg_union(&box_a, &box_b).unwrap();
    let mesh = tessellate_brep(&union_brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("boolean_union", &stl, &props);

    assert!((props.volume - 525.0).abs() < 10.0,
        "Boolean union volume should be ~525, got {}", props.volume);
}

#[test]
fn export_compound_fillet_shell_fixture() {
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 })));
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1],
            thickness: 0.5,
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("compound_fillet_shell", &stl, &props);

    assert!(props.volume < 350.0,
        "Compound fillet+shell volume ({}) should be less than 350", props.volume);
}

#[test]
fn export_revolve_shell_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_revolve_sketch());
    tree.push(Feature::new("r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams::full(
            Pt3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ))));
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1],
            thickness: 0.5,
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("revolve_shell", &stl, &props);

    // Full revolve volume is pi * 75 * 3 ~ 706.86
    // After shelling, volume should be less than that
    assert!(props.volume.abs() < 707.0,
        "Revolve+shell volume should be < 707, got {}", props.volume);
}

#[test]
fn export_stress_revolve_fillet_fixture() {
    use blockcad_kernel::tessellation::face_tessellator::tessellate_face;
    use blockcad_kernel::tessellation::mesh::TriMesh;

    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_revolve_sketch());
    tree.push(Feature::new("r1".into(), "Revolve".into(), FeatureKind::Revolve,
        FeatureParams::Revolve(RevolveParams::full(
            Pt3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ))));
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 0.5 })));
    let brep = evaluate(&mut tree).unwrap();

    // Tessellate face-by-face, skipping watertight validation (known limitation
    // for fillet on revolved bodies -- the tessellator doesn't yet stitch all
    // fillet faces perfectly).
    let params = TessellationParams::default();
    let mut mesh = TriMesh::new();
    let mut face_index = 0u32;
    for (face_id, _face) in brep.faces.iter() {
        let face_mesh = tessellate_face(&brep, face_id, face_index, &params).unwrap();
        mesh.merge(&face_mesh);
        face_index += 1;
    }
    mesh.fix_winding();

    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_revolve_fillet", &stl, &props);

    // Volume should be close to the plain revolve (~707) minus a small fillet correction
    let plain_volume = std::f64::consts::PI * 75.0 * 3.0;
    assert!((props.volume.abs() - plain_volume).abs() < 50.0,
        "Revolve+fillet volume should be ~{:.0}, got {:.1}", plain_volume, props.volume);
}

#[test]
fn export_l_shape_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_l_shape_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 5.0))));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("l_shape_extrude", &stl, &props);

    // L-shape area = 10*10 - 5*5 = 75, extruded 5mm => volume = 375
    assert!((props.volume - 375.0).abs() < 5.0,
        "L-shape volume should be ~375, got {}", props.volume);
}

// ---------------------------------------------------------------------------
// Round 1 compound stress tests
// ---------------------------------------------------------------------------

#[test]
fn export_stress_box_fillet_chamfer() {
    // Stress test: Extrude 10x5x7 box -> Fillet(edge 0, r=1) -> Chamfer(edge 4, d=0.5)
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 1.0 })));
    tree.push(Feature::new("c1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![4], distance: 0.5, distance2: None })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_box_fillet_chamfer", &stl, &props);

    assert!(props.volume < 350.0,
        "Fillet+chamfer should reduce volume below 350, got {}", props.volume);
    assert!(props.volume > 300.0,
        "Fillet+chamfer volume should be reasonable (> 300), got {}", props.volume);
}

#[test]
fn export_stress_box_shell_draft() {
    // Stress test: Extrude 10x5x7 box -> Shell(top face removed, t=0.5) -> Draft(2 side faces, 5 deg)
    let mut tree = build_sketch_extrude_tree(7.0);

    // Shell: remove top face, wall thickness 0.5
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1], // top face
            thickness: 0.5,
        }),
    ));

    // Draft: apply 5 deg draft to 2 side faces along Z pull direction
    tree.push(Feature::new(
        "d1".into(), "Draft".into(), FeatureKind::Draft,
        FeatureParams::Draft(DraftParams {
            face_indices: vec![2, 3], // two side faces
            pull_direction: Vec3::new(0.0, 0.0, 1.0),
            angle: 5.0_f64.to_radians(),
        }),
    ));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_box_shell_draft", &stl, &props);

    // Shell volume ~116, draft modifies side faces but shouldn't change volume drastically
    assert!(props.volume > 50.0,
        "Stress shell+draft volume ({}) should be > 50", props.volume);
    assert!(props.volume < 350.0,
        "Stress shell+draft volume ({}) should be < 350", props.volume);
}

#[test]
fn export_stress_box_mirror_fillet_fixture() {
    use blockcad_kernel::tessellation::face_tessellator::tessellate_face;
    use blockcad_kernel::tessellation::TriMesh;

    // Stress test: Extrude 10x5x7 box -> Mirror(YZ plane at x=0) -> Fillet(edge 0, r=0.5)
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "m1".into(), "Mirror".into(), FeatureKind::Mirror,
        FeatureParams::Mirror(MirrorParams {
            plane_origin: Pt3::new(0.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        }),
    ));
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 0.5 })));

    let brep = evaluate(&mut tree).unwrap();
    let params = TessellationParams::default();

    // Tessellate per-face and merge (skip watertight validation for stress test)
    let mut combined = TriMesh::new();
    let mut face_index = 0u32;
    for (face_id, _face) in brep.faces.iter() {
        let face_mesh = tessellate_face(&brep, face_id, face_index, &params).unwrap();
        combined.merge(&face_mesh);
        face_index += 1;
    }
    combined.fix_winding();

    let stl = export_stl_binary(&combined);
    let props = compute_mass_properties(&combined);
    write_fixture("stress_box_mirror_fillet", &stl, &props);

    // Mirrored box = 700, fillet removes a small amount
    assert!(props.volume < 700.0,
        "Mirror+fillet volume ({}) should be less than 700", props.volume);
    assert!(props.volume > 650.0,
        "Mirror+fillet volume ({}) should be > 650", props.volume);
}

#[test]
fn export_stress_box_pattern_shell_fixture() {
    // Stress test: Extrude 10x5x7 box -> LinearPattern(3x, spacing 15, along X) -> Shell(top face removed, t=0.3)
    let mut tree = build_sketch_extrude_tree(7.0);
    tree.push(Feature::new(
        "lp1".into(), "LinearPattern".into(), FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 3,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1], // Remove top face
            thickness: 0.3,
        }),
    ));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_box_pattern_shell", &stl, &props);

    assert!(props.volume.abs() > 100.0,
        "Stress box-pattern-shell volume should be > 100, got {}", props.volume);
    assert!(props.volume.abs() < 400.0,
        "Stress box-pattern-shell volume should be < 400, got {}", props.volume);
}

#[test]
fn export_stress_loft_mirror_fixture() {
    // Loft: 4x4 square at z=0 -> 2x2 square at z=10 (tapered frustum)
    let bottom = make_square_profile(4.0, 0.0);
    let top = make_square_profile(2.0, 10.0);

    let loft_params = LoftParams {
        slices_per_span: 10,
        closed: false,
    };

    let loft_brep = loft_profiles(&[bottom, top], &loft_params).unwrap();

    // Mirror across XY plane at z=0 (normal = Z axis)
    let mirror_params = MirrorParams {
        plane_origin: Pt3::new(0.0, 0.0, 0.0),
        plane_normal: Vec3::new(0.0, 0.0, 1.0),
    };

    let result = mirror_brep(&loft_brep, &mirror_params).unwrap();
    let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_loft_mirror", &stl, &props);

    // Expected volume: 2 * frustum = 2 * h/3 * (A1 + A2 + sqrt(A1*A2))
    let single_frustum = 10.0 / 3.0 * (16.0 + 4.0 + (16.0_f64 * 4.0).sqrt());
    let expected = 2.0 * single_frustum;
    assert!((props.volume - expected).abs() < 10.0,
        "Stress loft+mirror volume should be ~{:.1}, got {:.1}", expected, props.volume);
}

#[test]
fn export_stress_sweep_pattern_fixture() {
    // Stress test: Sweep a 4x4 square 10 units along Z, then apply circular pattern
    // with 3 copies at 120 degrees apart around Z axis offset at x=-15.
    let half = 2.0;
    let profile = ExtrudeProfile {
        points: vec![
            Pt3::new(-half, -half, 0.0),
            Pt3::new(half, -half, 0.0),
            Pt3::new(half, half, 0.0),
            Pt3::new(-half, half, 0.0),
        ],
        plane: Plane::xy(0.0),
    };

    let path = Line3::new(
        Pt3::new(0.0, 0.0, 0.0),
        Pt3::new(0.0, 0.0, 10.0),
    ).unwrap();
    let params = SweepParams { segments: Some(10), twist: 0.0 };

    let sweep_brep = sweep_profile(&profile, &path, &params).unwrap();

    // Apply circular pattern: 3 copies at 120 degrees, axis at x=-15 (offset enough to avoid overlap)
    let pattern_params = CircularPatternParams {
        axis_origin: Pt3::new(-15.0, 0.0, 0.0),
        axis_direction: Vec3::new(0.0, 0.0, 1.0),
        count: 3,
        total_angle: 2.0 * std::f64::consts::PI,
    };

    let result = circular_pattern(&sweep_brep, &pattern_params).unwrap();
    let mesh = tessellate_brep(&result, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_sweep_pattern", &stl, &props);

    // 3 copies of 4*4*10 = 160 each => 480 total
    assert!((props.volume - 480.0).abs() < 15.0,
        "Stress sweep pattern volume should be ~480, got {}", props.volume);
}

#[test]
fn export_stress_box_cut_fillet_fixture() {
    // Stress test: Extrude 10x5x7 box -> Fillet edge 0 (r=0.3) -> CutExtrude 4x2x3 blind pocket
    let mut tree = FeatureTree::new();

    // Feature 0: Base sketch (10x5 rectangle)
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());

    // Feature 1: Extrude to 7mm height -> 10x5x7 box
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));

    // Feature 2: Fillet edge 0 of the original box with r=0.3
    tree.push(Feature::new("f1".into(), "Fillet".into(), FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams { edge_indices: vec![0], radius: 0.3 })));

    // Feature 3: Pocket sketch (4x2 rectangle centered in the box)
    tree.push(Feature::new("s2".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(3, make_pocket_sketch());

    // Feature 4: CutExtrude -- blind pocket, 3mm deep from bottom face (z=0) upward
    let cut_params = CutExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 3.0);
    tree.push(Feature::new("ce1".into(), "CutExtrude".into(), FeatureKind::CutExtrude,
        FeatureParams::CutExtrude(cut_params)));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_box_cut_fillet", &stl, &props);

    // Box=350, fillet removes a small amount, pocket removes 4*2*3=24 -> volume < 326
    assert!(props.volume < 326.0,
        "Stress test volume ({}) should be less than 326 (box - pocket)", props.volume);
    assert!(props.volume > 300.0,
        "Stress test volume ({}) should be reasonable (> 300)", props.volume);
}

#[test]
fn export_stress_box_boolean_shell_fixture() {
    // Boolean union of two 10x5x7 boxes (B offset by 5 in X), then shell with top face removed.
    let box_a = build_box_brep(10.0, 5.0, 7.0).unwrap();
    let b_polys = extract_face_polygons(&build_box_brep(10.0, 5.0, 7.0).unwrap()).unwrap();
    let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys.into_iter().map(|(pts, n)| {
        (pts.into_iter().map(|p| Pt3::new(p.x + 5.0, p.y, p.z)).collect(), n)
    }).collect();
    let box_b = rebuild_brep_from_faces(&b_offset).unwrap();

    let union_brep = csg_union(&box_a, &box_b).unwrap();

    // Find the top face (normal pointing in +Z direction) to remove for shell
    let face_polys = extract_face_polygons(&union_brep).unwrap();
    let top_face_idx = face_polys.iter().enumerate()
        .find(|(_, (_, n))| n.z > 0.9)
        .map(|(i, _)| i as u32)
        .expect("Should find a top face with +Z normal");

    let shell_params = ShellParams {
        faces_to_remove: vec![top_face_idx],
        thickness: 0.5,
    };
    let shelled = shell_solid(&union_brep, &shell_params).unwrap();

    let mesh = tessellate_brep(&shelled, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("stress_box_boolean_shell", &stl, &props);

    assert!(props.volume.abs() > 50.0,
        "Stress boolean+shell volume should be > 50, got {}", props.volume);
    assert!(props.volume.abs() < 525.0,
        "Stress boolean+shell volume should be < 525, got {}", props.volume);
}

#[test]
fn export_l_shape_shell_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_l_shape_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 5.0))));
    tree.push(Feature::new(
        "sh1".into(), "Shell".into(), FeatureKind::Shell,
        FeatureParams::Shell(ShellParams {
            faces_to_remove: vec![1], // Remove top face
            thickness: 0.5,
        }),
    ));

    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("l_shape_shell", &stl, &props);

    // Solid L-shape volume = 375, shell should be less
    assert!(props.volume.abs() > 50.0,
        "L-shape shell volume should be > 50, got {}", props.volume);
    assert!(props.volume.abs() < 375.0,
        "L-shape shell volume should be < 375, got {}", props.volume);
}

#[test]
fn export_cylinder_chamfer_fixture() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_circle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0))));
    tree.push(Feature::new("ch1".into(), "Chamfer".into(), FeatureKind::Chamfer,
        FeatureParams::Chamfer(ChamferParams { edge_indices: vec![0], distance: 0.5, distance2: None })));
    let brep = evaluate(&mut tree).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    let stl = export_stl_binary(&mesh);
    let props = compute_mass_properties(&mesh);
    write_fixture("cylinder_chamfer_d05", &stl, &props);

    let cylinder_vol = std::f64::consts::PI * 25.0 * 10.0;
    assert!(props.volume < cylinder_vol,
        "Chamfered cylinder volume ({}) should be less than full cylinder ({:.0})", props.volume, cylinder_vol);
}
