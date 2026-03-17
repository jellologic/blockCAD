use criterion::{black_box, criterion_group, criterion_main, Criterion};

use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Vec3};
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::operations::pattern::linear::LinearPatternParams;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::ear_clip;
use blockcad_kernel::tessellation::mesh::TriMesh;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_rectangle_sketch() -> Sketch {
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 0.0) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(10.0, 5.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 5.0) });
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

fn build_box_tree(depth: f64) -> FeatureTree {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "s1".into(),
        "Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new(
        "e1".into(),
        "Extrude".into(),
        FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), depth)),
    ));
    tree
}

fn build_filleted_box_tree() -> FeatureTree {
    let mut tree = build_box_tree(7.0);
    tree.push(Feature::new(
        "f1".into(),
        "Fillet".into(),
        FeatureKind::Fillet,
        FeatureParams::Fillet(FilletParams {
            edge_indices: vec![0],
            radius: 1.0,
        }),
    ));
    tree
}

fn build_patterned_tree() -> FeatureTree {
    let mut tree = build_box_tree(5.0);
    tree.push(Feature::new(
        "lp1".into(),
        "Linear Pattern".into(),
        FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 5,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));
    tree
}

/// Build a convex polygon (regular n-gon) for ear-clip benchmarks.
fn convex_polygon(n: usize) -> Vec<[f64; 2]> {
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        pts.push([angle.cos() * 10.0, angle.sin() * 10.0]);
    }
    pts
}

/// Build a concave (star-like) polygon with `n` vertices.
fn concave_polygon(n: usize) -> Vec<[f64; 2]> {
    let mut pts = Vec::with_capacity(n);
    for i in 0..n {
        let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
        let r = if i % 2 == 0 { 10.0 } else { 5.0 };
        pts.push([angle.cos() * r, angle.sin() * r]);
    }
    pts
}

/// Build a watertight box mesh with the given number of triangles
/// (approximately). We tessellate a real box to get a valid mesh.
fn build_large_watertight_mesh(target_tris: usize) -> TriMesh {
    // A single box gives 12 triangles. We merge many boxes to reach the target.
    let copies = (target_tris / 12).max(1);
    let mut tree = build_box_tree(5.0);
    tree.push(Feature::new(
        "lp1".into(),
        "Linear Pattern".into(),
        FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: copies as u32,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));
    let brep = evaluate(&mut tree).expect("evaluate");
    tessellate_brep(&brep, &TessellationParams::default()).expect("tessellate")
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_tessellate_box(c: &mut Criterion) {
    let mut tree = build_box_tree(7.0);
    let brep = evaluate(&mut tree).expect("evaluate box");
    let params = TessellationParams::default();

    c.bench_function("tessellate_box", |b| {
        b.iter(|| {
            let mesh = tessellate_brep(black_box(&brep), black_box(&params)).unwrap();
            black_box(mesh);
        });
    });
}

fn bench_tessellate_filleted_box(c: &mut Criterion) {
    let mut tree = build_filleted_box_tree();
    let brep = evaluate(&mut tree).expect("evaluate filleted box");
    let params = TessellationParams::default();

    c.bench_function("tessellate_filleted_box", |b| {
        b.iter(|| {
            let mesh = tessellate_brep(black_box(&brep), black_box(&params)).unwrap();
            black_box(mesh);
        });
    });
}

fn bench_tessellate_patterned(c: &mut Criterion) {
    let mut tree = build_patterned_tree();
    let brep = evaluate(&mut tree).expect("evaluate patterned");
    let params = TessellationParams::default();

    c.bench_function("tessellate_patterned", |b| {
        b.iter(|| {
            let mesh = tessellate_brep(black_box(&brep), black_box(&params)).unwrap();
            black_box(mesh);
        });
    });
}

fn bench_ear_clip_simple(c: &mut Criterion) {
    let polygon = convex_polygon(10);

    c.bench_function("ear_clip_convex_10", |b| {
        b.iter(|| {
            let tris = ear_clip::triangulate(black_box(&polygon));
            black_box(tris);
        });
    });
}

fn bench_ear_clip_complex(c: &mut Criterion) {
    let polygon = concave_polygon(50);

    c.bench_function("ear_clip_concave_50", |b| {
        b.iter(|| {
            let tris = ear_clip::triangulate(black_box(&polygon));
            black_box(tris);
        });
    });
}

fn bench_watertight_check(c: &mut Criterion) {
    let mesh = build_large_watertight_mesh(1000);

    c.bench_function("watertight_check_1000tri", |b| {
        b.iter(|| {
            let result = black_box(&mesh).is_watertight();
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_tessellate_box,
    bench_tessellate_filleted_box,
    bench_tessellate_patterned,
    bench_ear_clip_simple,
    bench_ear_clip_complex,
    bench_watertight_check,
);
criterion_main!(benches);
