use crate::tessellation::mesh::TriMesh;
use std::fmt::Write;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ObjOptions {
    /// Decimal places for coordinates (default: 6)
    #[serde(default = "default_precision")]
    pub precision: u8,
}

fn default_precision() -> u8 { 6 }

impl Default for ObjOptions {
    fn default() -> Self {
        Self { precision: 6 }
    }
}

/// Export a TriMesh as Wavefront OBJ string with configurable precision.
pub fn export_obj(mesh: &TriMesh, object_name: &str, options: &ObjOptions) -> String {
    let vc = mesh.vertex_count();
    let tc = mesh.triangle_count();
    let has_uvs = !mesh.uvs.is_empty();
    let prec = options.precision as usize;

    let mut out = String::with_capacity(64 + 130 * vc + 40 * tc);

    let _ = writeln!(out, "# blockCAD OBJ export");
    let _ = writeln!(out, "o {}", object_name);

    for i in 0..vc {
        let base = i * 3;
        let _ = writeln!(out, "v {:.prec$} {:.prec$} {:.prec$}",
            mesh.positions[base], mesh.positions[base + 1], mesh.positions[base + 2], prec = prec);
    }

    for i in 0..vc {
        let base = i * 3;
        let _ = writeln!(out, "vn {:.prec$} {:.prec$} {:.prec$}",
            mesh.normals[base], mesh.normals[base + 1], mesh.normals[base + 2], prec = prec);
    }

    if has_uvs {
        let uv_count = mesh.uvs.len() / 2;
        for i in 0..uv_count {
            let base = i * 2;
            let _ = writeln!(out, "vt {:.prec$} {:.prec$}", mesh.uvs[base], mesh.uvs[base + 1], prec = prec);
        }
    }

    for tri in mesh.indices.chunks(3) {
        let i0 = tri[0] as usize + 1;
        let i1 = tri[1] as usize + 1;
        let i2 = tri[2] as usize + 1;

        if has_uvs {
            let _ = writeln!(out, "f {}/{}/{} {}/{}/{} {}/{}/{}", i0, i0, i0, i1, i1, i1, i2, i2, i2);
        } else {
            let _ = writeln!(out, "f {}//{} {}//{} {}//{}", i0, i0, i1, i1, i2, i2);
        }
    }

    out
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
        }
    }

    #[test]
    fn obj_single_triangle() {
        let text = export_obj(&simple_triangle(), "test", &ObjOptions::default());
        assert!(text.contains("o test"));
        assert_eq!(text.matches("\nv ").count(), 3);
        assert_eq!(text.matches("\nvn ").count(), 3);
        assert_eq!(text.matches("\nf ").count(), 1);
        assert!(text.contains("1/1/1"));
    }

    #[test]
    fn obj_without_uvs() {
        let mesh = TriMesh {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            uvs: vec![], indices: vec![0, 1, 2], face_ids: vec![0], colors: vec![],
        };
        let text = export_obj(&mesh, "test", &ObjOptions::default());
        assert!(text.contains("1//1"));
    }

    #[test]
    fn obj_precision_2() {
        let text = export_obj(&simple_triangle(), "test", &ObjOptions { precision: 2 });
        assert!(text.contains("0.00 ") || text.contains("0.00\n"));
        assert!(!text.contains("0.000000"));
    }

    #[test]
    fn obj_empty_mesh() {
        let text = export_obj(&TriMesh::new(), "empty", &ObjOptions::default());
        assert!(text.contains("o empty"));
        assert_eq!(text.matches("\nv ").count(), 0);
    }
}
