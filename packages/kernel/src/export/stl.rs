use crate::tessellation::mesh::TriMesh;
use std::fmt::Write;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StlOptions {
    /// Decimal places for ASCII coordinates (default: 6)
    #[serde(default = "default_precision")]
    pub precision: u8,
}

fn default_precision() -> u8 { 6 }

impl Default for StlOptions {
    fn default() -> Self {
        Self { precision: 6 }
    }
}

/// Export a TriMesh as binary STL bytes.
///
/// Binary STL always uses f32 — precision option does not apply.
pub fn export_stl_binary(mesh: &TriMesh) -> Vec<u8> {
    let tri_count = mesh.triangle_count() as u32;
    let mut buf = Vec::with_capacity(84 + 50 * tri_count as usize);

    let mut header = [0u8; 80];
    let label = b"blockCAD STL export";
    header[..label.len()].copy_from_slice(label);
    buf.extend_from_slice(&header);

    buf.extend_from_slice(&tri_count.to_le_bytes());

    for tri in mesh.indices.chunks(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let v0 = get_vertex(&mesh.positions, i0);
        let v1 = get_vertex(&mesh.positions, i1);
        let v2 = get_vertex(&mesh.positions, i2);

        let (nx, ny, nz) = face_normal(&v0, &v1, &v2);

        buf.extend_from_slice(&nx.to_le_bytes());
        buf.extend_from_slice(&ny.to_le_bytes());
        buf.extend_from_slice(&nz.to_le_bytes());

        for v in &[v0, v1, v2] {
            buf.extend_from_slice(&v[0].to_le_bytes());
            buf.extend_from_slice(&v[1].to_le_bytes());
            buf.extend_from_slice(&v[2].to_le_bytes());
        }

        buf.extend_from_slice(&0u16.to_le_bytes());
    }

    buf
}

/// Export a TriMesh as ASCII STL string with configurable precision.
pub fn export_stl_ascii(mesh: &TriMesh, solid_name: &str, options: &StlOptions) -> String {
    let tri_count = mesh.triangle_count();
    let prec = options.precision as usize;
    let mut out = String::with_capacity(64 + 200 * tri_count);

    let _ = writeln!(out, "solid {}", solid_name);

    for tri in mesh.indices.chunks(3) {
        let v0 = get_vertex(&mesh.positions, tri[0] as usize);
        let v1 = get_vertex(&mesh.positions, tri[1] as usize);
        let v2 = get_vertex(&mesh.positions, tri[2] as usize);

        let (nx, ny, nz) = face_normal(&v0, &v1, &v2);

        let _ = writeln!(out, "  facet normal {:.prec$} {:.prec$} {:.prec$}", nx, ny, nz, prec = prec);
        let _ = writeln!(out, "    outer loop");
        let _ = writeln!(out, "      vertex {:.prec$} {:.prec$} {:.prec$}", v0[0], v0[1], v0[2], prec = prec);
        let _ = writeln!(out, "      vertex {:.prec$} {:.prec$} {:.prec$}", v1[0], v1[1], v1[2], prec = prec);
        let _ = writeln!(out, "      vertex {:.prec$} {:.prec$} {:.prec$}", v2[0], v2[1], v2[2], prec = prec);
        let _ = writeln!(out, "    endloop");
        let _ = writeln!(out, "  endfacet");
    }

    let _ = writeln!(out, "endsolid {}", solid_name);
    out
}

fn get_vertex(positions: &[f32], index: usize) -> [f32; 3] {
    let base = index * 3;
    [positions[base], positions[base + 1], positions[base + 2]]
}

fn face_normal(v0: &[f32; 3], v1: &[f32; 3], v2: &[f32; 3]) -> (f32, f32, f32) {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len > 1e-12 { (nx / len, ny / len, nz / len) } else { (0.0, 0.0, 1.0) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_triangle() -> TriMesh {
        TriMesh {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            uvs: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            indices: vec![0, 1, 2],
            face_ids: vec![0],
            colors: vec![],
            ..Default::default()
        }
    }

    #[test]
    fn stl_binary_single_triangle() {
        let mesh = simple_triangle();
        let bytes = export_stl_binary(&mesh);
        assert_eq!(bytes.len(), 134);
        let tc = u32::from_le_bytes([bytes[80], bytes[81], bytes[82], bytes[83]]);
        assert_eq!(tc, 1);
    }

    #[test]
    fn stl_binary_empty_mesh() {
        let mesh = TriMesh::new();
        let bytes = export_stl_binary(&mesh);
        assert_eq!(bytes.len(), 84);
    }

    #[test]
    fn stl_ascii_default_precision() {
        let mesh = simple_triangle();
        let text = export_stl_ascii(&mesh, "test", &StlOptions::default());
        assert!(text.starts_with("solid test"));
        assert!(text.contains("endsolid test"));
        // Default precision 6: should have 6 decimal places
        assert!(text.contains("0.000000"));
    }

    #[test]
    fn stl_ascii_precision_3() {
        let mesh = simple_triangle();
        let text = export_stl_ascii(&mesh, "test", &StlOptions { precision: 3 });
        // Should have 3 decimal places, not 6
        assert!(text.contains("0.000 ") || text.contains("0.000\n"));
        assert!(!text.contains("0.000000"));
    }

    #[test]
    fn stl_ascii_empty_mesh() {
        let mesh = TriMesh::new();
        let text = export_stl_ascii(&mesh, "empty", &StlOptions::default());
        assert!(text.starts_with("solid empty"));
        assert!(text.contains("endsolid empty"));
    }
}
