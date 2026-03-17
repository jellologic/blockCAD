use blockcad_kernel::tessellation::mesh::TriMesh;
use blockcad_kernel::tessellation::params::TessellationParams;

#[test]
fn empty_mesh_is_valid() {
    let mesh = TriMesh::new();
    assert!(mesh.validate().is_ok());
    assert_eq!(mesh.vertex_count(), 0);
    assert_eq!(mesh.triangle_count(), 0);
}

#[test]
fn valid_quad_mesh() {
    // An open quad (two triangles) is not watertight, so validate() returns Err
    let mesh = TriMesh {
        positions: vec![
            0.0, 0.0, 0.0, // v0
            1.0, 0.0, 0.0, // v1
            1.0, 1.0, 0.0, // v2
            0.0, 1.0, 0.0, // v3
        ],
        normals: vec![
            0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0,
        ],
        uvs: vec![0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0],
        indices: vec![0, 1, 2, 0, 2, 3],
        face_ids: vec![0, 0],
        colors: vec![],
        ..Default::default()
    };
    assert!(mesh.validate().is_err()); // not watertight
    assert_eq!(mesh.vertex_count(), 4);
    assert_eq!(mesh.triangle_count(), 2);
}

#[test]
fn out_of_bounds_index_fails() {
    let mesh = TriMesh {
        positions: vec![0.0; 9], // 3 vertices
        normals: vec![0.0; 9],
        uvs: vec![0.0; 6],
        indices: vec![0, 1, 5], // index 5 is out of bounds
        face_ids: vec![0],
        colors: vec![],
        ..Default::default()
    };
    assert!(mesh.validate().is_err());
}

#[test]
fn mismatched_normals_fails() {
    let mesh = TriMesh {
        positions: vec![0.0; 9],
        normals: vec![0.0; 6], // should be 9
        uvs: vec![0.0; 6],
        indices: vec![0, 1, 2],
        face_ids: vec![0],
        colors: vec![],
        ..Default::default()
    };
    assert!(mesh.validate().is_err());
}

#[test]
fn tessellation_params_presets() {
    let default = TessellationParams::default();
    let hq = TessellationParams::high_quality();
    let preview = TessellationParams::preview();

    assert!(hq.chord_tolerance < default.chord_tolerance);
    assert!(preview.chord_tolerance > default.chord_tolerance);
}

#[test]
fn mesh_merge_preserves_structure() {
    let a = TriMesh {
        positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
        normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        uvs: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
        indices: vec![0, 1, 2],
        face_ids: vec![0],
        colors: vec![],
        ..Default::default()
    };
    let b = a.clone();
    let mut merged = a;
    merged.merge(&b);
    // Two open triangles are not watertight, but merge itself works correctly
    assert_eq!(merged.triangle_count(), 2);
    assert_eq!(merged.vertex_count(), 6);
}
