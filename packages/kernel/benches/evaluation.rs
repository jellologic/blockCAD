use criterion::{black_box, criterion_group, criterion_main, Criterion};

use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Vec3};
use blockcad_kernel::operations::chamfer::ChamferParams;
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::operations::fillet::FilletParams;
use blockcad_kernel::operations::pattern::linear::LinearPatternParams;
use blockcad_kernel::operations::shell::{ShellDirection, ShellParams};
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
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

/// Build a feature tree with a given number of features.
/// Layout:
///   0: Sketch
///   1: Extrude
///   2: Fillet (edge 0)
///   3: Chamfer (edge 4)
///   4: Shell (face 0)
///   5+: Linear patterns with increasing count
fn build_n_feature_tree(n: usize) -> FeatureTree {
    assert!(n >= 2, "need at least sketch + extrude");
    let mut tree = FeatureTree::new();

    // Feature 0: Sketch
    tree.push(Feature::new(
        "s1".into(),
        "Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.sketches.insert(0, make_rectangle_sketch());

    // Feature 1: Extrude
    tree.push(Feature::new(
        "e1".into(),
        "Extrude".into(),
        FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0)),
    ));

    if n <= 2 {
        return tree;
    }

    // Feature 2: Fillet
    if n > 2 {
        tree.push(Feature::new(
            "f1".into(),
            "Fillet".into(),
            FeatureKind::Fillet,
            FeatureParams::Fillet(FilletParams {
                edge_indices: vec![0],
                radius: 0.5,
            }),
        ));
    }

    // Feature 3: Chamfer
    if n > 3 {
        tree.push(Feature::new(
            "ch1".into(),
            "Chamfer".into(),
            FeatureKind::Chamfer,
            FeatureParams::Chamfer(ChamferParams {
                edge_indices: vec![4],
                distance: 0.5,
                distance2: None,
                mode: None,
            }),
        ));
    }

    // Feature 4: Shell
    if n > 4 {
        tree.push(Feature::new(
            "sh1".into(),
            "Shell".into(),
            FeatureKind::Shell,
            FeatureParams::Shell(ShellParams {
                faces_to_remove: vec![0],
                thickness: 0.5,
                direction: ShellDirection::Inward,
            }),
        ));
    }

    // Features 5..n: Linear patterns with count 2
    for i in 5..n {
        tree.push(Feature::new(
            format!("lp{}", i - 4),
            format!("Linear Pattern {}", i - 4),
            FeatureKind::LinearPattern,
            FeatureParams::LinearPattern(LinearPatternParams {
                direction: Vec3::new(1.0, 0.0, 0.0),
                spacing: 15.0,
                count: 2,
                direction2: None,
                spacing2: None,
                count2: None,
            }),
        ));
    }

    tree
}

// ---------------------------------------------------------------------------
// Benchmarks
// ---------------------------------------------------------------------------

fn bench_evaluate_5_features(c: &mut Criterion) {
    c.bench_function("evaluate_5_features", |b| {
        b.iter_batched(
            || build_n_feature_tree(5),
            |mut tree| {
                let brep = evaluate(black_box(&mut tree)).unwrap();
                black_box(brep);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_evaluate_10_features(c: &mut Criterion) {
    c.bench_function("evaluate_10_features", |b| {
        b.iter_batched(
            || build_n_feature_tree(10),
            |mut tree| {
                let brep = evaluate(black_box(&mut tree)).unwrap();
                black_box(brep);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_evaluate_20_features(c: &mut Criterion) {
    c.bench_function("evaluate_20_features", |b| {
        b.iter_batched(
            || build_n_feature_tree(20),
            |mut tree| {
                let brep = evaluate(black_box(&mut tree)).unwrap();
                black_box(brep);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Build a feature tree that produces a watertight mesh end-to-end:
/// sketch + extrude + linear pattern (3 copies).
fn build_pipeline_tree() -> FeatureTree {
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
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0)),
    ));

    tree.push(Feature::new(
        "lp1".into(),
        "Linear Pattern".into(),
        FeatureKind::LinearPattern,
        FeatureParams::LinearPattern(LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 15.0,
            count: 3,
            direction2: None,
            spacing2: None,
            count2: None,
        }),
    ));

    tree
}

fn bench_full_pipeline(c: &mut Criterion) {
    let params = TessellationParams::default();

    c.bench_function("full_pipeline_evaluate_tessellate", |b| {
        b.iter_batched(
            || build_pipeline_tree(),
            |mut tree| {
                let brep = evaluate(black_box(&mut tree)).unwrap();
                let mesh = tessellate_brep(&brep, &params).unwrap();
                black_box(mesh);
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    bench_evaluate_5_features,
    bench_evaluate_10_features,
    bench_evaluate_20_features,
    bench_full_pipeline,
);
criterion_main!(benches);
