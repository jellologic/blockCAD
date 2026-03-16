//! Export validation tests using third-party parsers.
//!
//! Each test builds a known model (10×5×7 box), exports it,
//! then parses the output with an established third-party crate
//! to prove interoperability — not just self-consistency.
//!
//! Third-party parsers used:
//! - stl_io: Standard STL parser (binary + ASCII)
//! - tobj: Wavefront OBJ parser (tinyobjloader port)
//! - gltf: Khronos-official glTF/GLB parser
//! - zip: Standard ZIP archive reader (for 3MF)

use blockcad_kernel::export;
use blockcad_kernel::feature_tree::evaluator::evaluate;
use blockcad_kernel::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::{Pt2, Vec3};
use blockcad_kernel::operations::extrude::ExtrudeParams;
use blockcad_kernel::sketch::constraint::{Constraint, ConstraintKind};
use blockcad_kernel::sketch::entity::SketchEntity;
use blockcad_kernel::sketch::Sketch;
use blockcad_kernel::tessellation::mesh::TriMesh;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};

/// Expected box dimensions: 10 × 5 × 7
const BOX_MIN: [f32; 3] = [0.0, 0.0, 0.0];
const BOX_MAX: [f32; 3] = [10.0, 5.0, 7.0];
const EXPECTED_VERTICES: usize = 24;
const EXPECTED_TRIANGLES: usize = 12;

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

fn build_box_mesh() -> TriMesh {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new("s1".into(), "Sketch".into(), FeatureKind::Sketch, FeatureParams::Placeholder));
    tree.sketches.insert(0, make_rectangle_sketch());
    tree.push(Feature::new("e1".into(), "Extrude".into(), FeatureKind::Extrude,
        FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 7.0))));
    let brep = evaluate(&mut tree).unwrap();
    tessellate_brep(&brep, &TessellationParams::default()).unwrap()
}

fn assert_bounds(min: [f32; 3], max: [f32; 3], label: &str) {
    let tol = 0.1;
    for i in 0..3 {
        assert!((min[i] - BOX_MIN[i]).abs() < tol,
            "{}: min[{}] = {}, expected ~{}", label, i, min[i], BOX_MIN[i]);
        assert!((max[i] - BOX_MAX[i]).abs() < tol,
            "{}: max[{}] = {}, expected ~{}", label, i, max[i], BOX_MAX[i]);
    }
}

// ─── STL BINARY — parsed by stl_io ────────────────────────────

#[test]
fn stl_binary_parsed_by_stl_io() {
    let mesh = build_box_mesh();
    let bytes = export::stl::export_stl_binary(&mesh);

    // Parse with third-party stl_io crate
    let mut cursor = std::io::Cursor::new(&bytes);
    let stl = stl_io::read_stl(&mut cursor)
        .expect("stl_io failed to parse our binary STL output");

    assert_eq!(stl.faces.len(), EXPECTED_TRIANGLES,
        "stl_io parsed {} faces, expected {}", stl.faces.len(), EXPECTED_TRIANGLES);

    // Verify geometry from parsed data (IndexedMesh has vertices + faces)
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];

    // Verify normals are unit-length
    for face in &stl.faces {
        let n = face.normal;
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 0.01,
            "stl_io: face normal not unit length: {:.4}", len);
    }

    // Track bounds from all vertices
    for v in &stl.vertices {
        for i in 0..3 {
            if v[i] < min[i] { min[i] = v[i]; }
            if v[i] > max[i] { max[i] = v[i]; }
        }
    }

    assert_bounds(min, max, "stl_io binary");
}

// ─── STL ASCII — parsed by stl_io ─────────────────────────────

#[test]
fn stl_ascii_parsed_by_stl_io() {
    let mesh = build_box_mesh();
    let text = export::stl::export_stl_ascii(&mesh, "test_box", &export::StlOptions::default());

    // stl_io auto-detects ASCII vs binary
    let mut cursor = std::io::Cursor::new(text.as_bytes());
    let stl = stl_io::read_stl(&mut cursor)
        .expect("stl_io failed to parse our ASCII STL output");

    assert_eq!(stl.faces.len(), EXPECTED_TRIANGLES,
        "stl_io parsed {} faces from ASCII, expected {}", stl.faces.len(), EXPECTED_TRIANGLES);

    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for v in &stl.vertices {
        for i in 0..3 {
            if v[i] < min[i] { min[i] = v[i]; }
            if v[i] > max[i] { max[i] = v[i]; }
        }
    }
    assert_bounds(min, max, "stl_io ASCII");
}

// ─── OBJ — parsed by tobj ─────────────────────────────────────

#[test]
fn obj_parsed_by_tobj() {
    let mesh = build_box_mesh();
    let text = export::obj::export_obj(&mesh, "test_box", &export::ObjOptions::default());

    // Parse with third-party tobj crate
    let mut cursor = std::io::BufReader::new(std::io::Cursor::new(text.as_bytes()));
    let (models, _materials) = tobj::load_obj_buf(
        &mut cursor,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |_mtl_path| Ok(Default::default()),
    ).expect("tobj failed to parse our OBJ output");

    assert_eq!(models.len(), 1, "tobj: expected 1 model, got {}", models.len());

    let m = &models[0].mesh;
    let vertex_count = m.positions.len() / 3;
    let tri_count = m.indices.len() / 3;

    assert_eq!(vertex_count, EXPECTED_VERTICES,
        "tobj: parsed {} vertices, expected {}", vertex_count, EXPECTED_VERTICES);
    assert_eq!(tri_count, EXPECTED_TRIANGLES,
        "tobj: parsed {} triangles, expected {}", tri_count, EXPECTED_TRIANGLES);

    // Verify all indices are in bounds
    for &idx in &m.indices {
        assert!((idx as usize) < vertex_count,
            "tobj: index {} out of bounds (vertex count: {})", idx, vertex_count);
    }

    // Verify normals were parsed
    let normal_count = m.normals.len() / 3;
    assert_eq!(normal_count, EXPECTED_VERTICES,
        "tobj: parsed {} normals, expected {}", normal_count, EXPECTED_VERTICES);

    // Verify bounds from parsed positions
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for i in 0..vertex_count {
        let base = i * 3;
        for j in 0..3 {
            let v = m.positions[base + j];
            if v < min[j] { min[j] = v; }
            if v > max[j] { max[j] = v; }
        }
    }
    assert_bounds(min, max, "tobj OBJ");
}

// ─── 3MF — validated by zip crate (third-party) ───────────────

#[test]
fn threemf_parsed_by_zip() {
    let mesh = build_box_mesh();
    let bytes = export::threemf::export_3mf(&mesh, "test_box", &export::ThreeMfOptions::default()).unwrap();

    // Parse with third-party zip crate
    let cursor = std::io::Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor)
        .expect("zip crate failed to parse our 3MF output");

    // Verify required 3MF files exist
    let names: Vec<String> = archive.file_names().map(String::from).collect();
    assert!(names.contains(&"[Content_Types].xml".to_string()), "Missing [Content_Types].xml");
    assert!(names.contains(&"_rels/.rels".to_string()), "Missing _rels/.rels");
    assert!(names.contains(&"3D/3dmodel.model".to_string()), "Missing 3D/3dmodel.model");

    // Extract and validate model XML
    let mut model_file = archive.by_name("3D/3dmodel.model")
        .expect("zip: failed to extract 3D/3dmodel.model");
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut model_file, &mut xml).unwrap();

    // Parse vertices and triangles from XML
    let vertex_count = xml.lines()
        .filter(|l| l.trim().starts_with("<vertex"))
        .count();
    let tri_count = xml.lines()
        .filter(|l| l.trim().starts_with("<triangle "))
        .count();

    assert_eq!(vertex_count, EXPECTED_VERTICES,
        "3MF: {} vertices, expected {}", vertex_count, EXPECTED_VERTICES);
    assert_eq!(tri_count, EXPECTED_TRIANGLES,
        "3MF: {} triangles, expected {}", tri_count, EXPECTED_TRIANGLES);

    // Validate XML structure contains 3MF namespace
    assert!(xml.contains("http://schemas.microsoft.com/3dmanufacturing/core/2015/02"),
        "3MF: missing 3MF namespace in model XML");
    assert!(xml.contains("<mesh>"), "3MF: missing <mesh> element");
    assert!(xml.contains("<vertices>"), "3MF: missing <vertices> element");
    assert!(xml.contains("<triangles>"), "3MF: missing <triangles> element");
}

// ─── GLB — parsed by gltf crate (Khronos-official) ────────────

#[test]
fn glb_parsed_by_gltf() {
    let mesh = build_box_mesh();
    let bytes = export::gltf::export_glb(&mesh, "test_box", &export::GlbOptions::default()).unwrap();

    // Parse with Khronos-official gltf crate — performs full spec validation
    let (document, buffers, _images) = gltf::import_slice(&bytes)
        .expect("gltf crate failed to parse our GLB output");

    // Verify scene structure
    assert_eq!(document.scenes().count(), 1, "gltf: expected 1 scene");
    assert_eq!(document.nodes().count(), 1, "gltf: expected 1 node");
    assert_eq!(document.meshes().count(), 1, "gltf: expected 1 mesh");

    // Verify mesh primitive
    let gltf_mesh = document.meshes().next().unwrap();
    let primitive = gltf_mesh.primitives().next().unwrap();
    assert_eq!(primitive.mode(), gltf::mesh::Mode::Triangles, "gltf: expected triangle mode");

    // Verify position accessor
    let pos_accessor = primitive.get(&gltf::Semantic::Positions).unwrap();
    assert_eq!(pos_accessor.count(), EXPECTED_VERTICES,
        "gltf: position accessor count {}, expected {}", pos_accessor.count(), EXPECTED_VERTICES);
    assert_eq!(pos_accessor.data_type(), gltf::accessor::DataType::F32);
    assert_eq!(pos_accessor.dimensions(), gltf::accessor::Dimensions::Vec3);

    // Verify normal accessor
    let norm_accessor = primitive.get(&gltf::Semantic::Normals).unwrap();
    assert_eq!(norm_accessor.count(), EXPECTED_VERTICES,
        "gltf: normal accessor count {}, expected {}", norm_accessor.count(), EXPECTED_VERTICES);

    // Verify index accessor
    let idx_accessor = primitive.indices().unwrap();
    assert_eq!(idx_accessor.count(), EXPECTED_TRIANGLES * 3,
        "gltf: index accessor count {}, expected {}", idx_accessor.count(), EXPECTED_TRIANGLES * 3);

    // Read actual position data from buffer and verify bounds
    let buffer_data = &buffers[0];
    let view = pos_accessor.view().unwrap();
    let offset = view.offset();
    let length = view.length();
    let pos_bytes = &buffer_data[offset..offset + length];

    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for i in 0..EXPECTED_VERTICES {
        for j in 0..3 {
            let byte_off = (i * 3 + j) * 4;
            let val = f32::from_le_bytes([
                pos_bytes[byte_off], pos_bytes[byte_off+1],
                pos_bytes[byte_off+2], pos_bytes[byte_off+3]
            ]);
            assert!(!val.is_nan(), "gltf: position NaN at vertex {} component {}", i, j);
            if val < min[j] { min[j] = val; }
            if val > max[j] { max[j] = val; }
        }
    }
    assert_bounds(min, max, "gltf GLB");

    // Verify accessor min/max match actual data
    if let Some(gltf_min) = pos_accessor.min() {
        if let serde_json::Value::Array(arr) = gltf_min {
            for i in 0..3 {
                if let Some(val) = arr[i].as_f64() {
                    assert!((val as f32 - min[i]).abs() < 0.01,
                        "gltf: accessor min[{}] = {}, actual min = {}", i, val, min[i]);
                }
            }
        }
    }
}

// ─── CROSS-FORMAT CONSISTENCY ──────────────────────────────────

#[test]
fn all_formats_agree_on_geometry() {
    let mesh = build_box_mesh();

    // STL via stl_io
    let stl_bytes = export::stl::export_stl_binary(&mesh);
    let stl = stl_io::read_stl(&mut std::io::Cursor::new(&stl_bytes)).unwrap();
    let stl_tris = stl.faces.len();

    // OBJ via tobj
    let obj_text = export::obj::export_obj(&mesh, "box", &export::ObjOptions::default());
    let (models, _) = tobj::load_obj_buf(
        &mut std::io::BufReader::new(std::io::Cursor::new(obj_text.as_bytes())),
        &tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() },
        |_| Ok(Default::default()),
    ).unwrap();
    let obj_verts = models[0].mesh.positions.len() / 3;
    let obj_tris = models[0].mesh.indices.len() / 3;

    // GLB via gltf
    let glb_bytes = export::gltf::export_glb(&mesh, "box", &export::GlbOptions::default()).unwrap();
    let (doc, _, _) = gltf::import_slice(&glb_bytes).unwrap();
    let gltf_mesh = doc.meshes().next().unwrap();
    let prim = gltf_mesh.primitives().next().unwrap();
    let glb_verts = prim.get(&gltf::Semantic::Positions).unwrap().count();
    let glb_indices = prim.indices().unwrap().count();

    // 3MF via zip
    let tmf_bytes = export::threemf::export_3mf(&mesh, "box", &export::ThreeMfOptions::default()).unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(tmf_bytes)).unwrap();
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut archive.by_name("3D/3dmodel.model").unwrap(), &mut xml).unwrap();
    let tmf_verts = xml.lines().filter(|l| l.trim().starts_with("<vertex")).count();
    let tmf_tris = xml.lines().filter(|l| l.trim().starts_with("<triangle ")).count();

    // All must agree
    assert_eq!(stl_tris, EXPECTED_TRIANGLES, "STL triangle count");
    assert_eq!(obj_tris, EXPECTED_TRIANGLES, "OBJ triangle count");
    assert_eq!(glb_indices / 3, EXPECTED_TRIANGLES, "GLB triangle count");
    assert_eq!(tmf_tris, EXPECTED_TRIANGLES, "3MF triangle count");

    assert_eq!(obj_verts, EXPECTED_VERTICES, "OBJ vertex count");
    assert_eq!(glb_verts, EXPECTED_VERTICES, "GLB vertex count");
    assert_eq!(tmf_verts, EXPECTED_VERTICES, "3MF vertex count");
}
