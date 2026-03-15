//! End-to-end vertical slice test:
//! Sketch a rectangle → Solve constraints → Extrude → Tessellate → Serialize

use blockcad_kernel::feature_tree::*;
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Pt3, Vec3};
use blockcad_kernel::operations::extrude::{extrude_profile, ExtrudeParams, ExtrudeProfile};
use blockcad_kernel::serialization::feature_tree_io;
use blockcad_kernel::serialization::schema::KernelDocument;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::solver::equations::*;
use blockcad_kernel::solver::graph::ConstraintGraph;
use blockcad_kernel::solver::newton_raphson::{solve, SolverConfig};
use blockcad_kernel::solver::variable::Variable;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams, TriMesh};
use blockcad_kernel::topology::builders::build_box_brep;

#[test]
fn full_pipeline_sketch_extrude_tessellate() {
    // === Step 1: Create sketch with rectangle ===
    let mut sketch = Sketch::new(Plane::xy(0.0));
    let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
    let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 0.5) });
    let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(8.0, 4.0) });
    let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.5, 4.0) });

    let _l0 = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
    let _l1 = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
    let _l2 = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
    let _l3 = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });

    assert_eq!(sketch.entity_count(), 8); // 4 points + 4 lines

    // === Step 2: Solve constraints ===
    let mut graph = ConstraintGraph::new();

    // Map sketch points to solver variables
    let x0 = graph.variables.add(Variable::fixed(0.0));
    let y0 = graph.variables.add(Variable::fixed(0.0));
    let x1 = graph.variables.add(Variable::new(8.0));
    let y1 = graph.variables.add(Variable::new(0.5));
    let x2 = graph.variables.add(Variable::new(8.0));
    let y2 = graph.variables.add(Variable::new(4.0));
    let x3 = graph.variables.add(Variable::new(0.5));
    let y3 = graph.variables.add(Variable::new(4.0));

    // Horizontal: p0-p1 (y0 == y1)
    graph.add_equation(Box::new(CoincidentEquation::new(y0, y1)));
    // Vertical: p1-p2 (x1 == x2)
    graph.add_equation(Box::new(CoincidentEquation::new(x1, x2)));
    // Horizontal: p2-p3 (y2 == y3)
    graph.add_equation(Box::new(CoincidentEquation::new(y2, y3)));
    // Vertical: p3-p0 (x3 == x0)
    graph.add_equation(Box::new(CoincidentEquation::new(x3, x0)));
    // Width = 10
    graph.add_equation(Box::new(DistanceEquation::new(x0, y0, x1, y1, 10.0)));
    // Height = 5
    graph.add_equation(Box::new(DistanceEquation::new(x1, y1, x2, y2, 5.0)));

    let result = solve(&mut graph, &SolverConfig::default()).unwrap();
    assert!(result.converged, "Constraint solver did not converge");

    // Read solved positions
    let corners = [
        Pt3::new(graph.variables.value(x0), graph.variables.value(y0), 0.0),
        Pt3::new(graph.variables.value(x1), graph.variables.value(y1), 0.0),
        Pt3::new(graph.variables.value(x2), graph.variables.value(y2), 0.0),
        Pt3::new(graph.variables.value(x3), graph.variables.value(y3), 0.0),
    ];

    // Verify rectangle: width=10, height=5
    let width = (corners[1] - corners[0]).norm();
    let height = (corners[2] - corners[1]).norm();
    assert!((width - 10.0).abs() < 1e-6, "Width: {}", width);
    assert!((height - 5.0).abs() < 1e-6, "Height: {}", height);

    // === Step 3: Extrude along Z ===
    let profile = ExtrudeProfile {
        points: corners.to_vec(),
        plane: Plane::xy(0.0),
    };
    let depth = 7.0;
    let brep = extrude_profile(&profile, Vec3::new(0.0, 0.0, 1.0), depth).unwrap();

    // Verify topology: box has 6 faces
    assert_eq!(brep.faces.len(), 6, "Extruded box should have 6 faces");

    // === Step 4: Tessellate ===
    let params = TessellationParams::default();
    let mesh = tessellate_brep(&brep, &params).unwrap();

    // 6 faces × 2 triangles = 12 triangles
    assert_eq!(mesh.triangle_count(), 12, "Box should have 12 triangles");
    assert!(mesh.validate().is_ok(), "Mesh should be valid");

    // Mesh has data that could be sent to GPU
    let bytes = mesh.to_bytes();
    assert!(bytes.len() > 0, "Mesh bytes should not be empty");

    // === Step 5: Serialize to .blockcad format ===
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "sketch-1".into(),
        "Base Sketch".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.push(Feature::new(
        "extrude-1".into(),
        "Extrude Base".into(),
        FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams {
            direction: Vec3::new(0.0, 0.0, 1.0),
            depth,
            symmetric: false,
            draft_angle: 0.0,
        }),
    ));

    let doc = feature_tree_io::serialize_tree(&tree, "Vertical Slice Box").unwrap();
    let json = doc.to_json_pretty().unwrap();

    // Verify JSON format
    assert!(json.contains(r#""name": "Vertical Slice Box""#));
    assert!(json.contains(r#""type": "extrude""#));
    assert!(json.contains(r#""depth": 7.0"#));

    // Roundtrip
    let doc2 = KernelDocument::from_json(&json).unwrap();
    assert_eq!(doc2.features.len(), 2);
    assert_eq!(doc2.features[0].id, "sketch-1");
    assert_eq!(doc2.features[1].id, "extrude-1");
}

#[test]
fn build_box_and_tessellate() {
    // Simpler test: build box directly and tessellate
    let brep = build_box_brep(5.0, 3.0, 2.0).unwrap();
    let mesh = tessellate_brep(&brep, &TessellationParams::default()).unwrap();
    assert_eq!(mesh.triangle_count(), 12);
    assert_eq!(mesh.vertex_count(), 24);
    assert!(mesh.validate().is_ok());
}
