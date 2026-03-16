use crate::error::{KernelError, KernelResult};
use crate::tessellation::mesh::TriMesh;
use std::fmt::Write as FmtWrite;
use std::io::{Cursor, Write};
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThreeMfOptions {
    /// Unit system: "millimeter", "centimeter", "meter", "inch", "foot" (default: "millimeter")
    #[serde(default = "default_unit")]
    pub unit: String,
    /// Include vertex colors if available in mesh (default: false)
    #[serde(default)]
    pub vertex_colors: bool,
}

fn default_unit() -> String { "millimeter".into() }

impl Default for ThreeMfOptions {
    fn default() -> Self {
        Self { unit: "millimeter".into(), vertex_colors: false }
    }
}

/// Export a TriMesh as 3MF bytes (ZIP archive containing XML model).
pub fn export_3mf(mesh: &TriMesh, model_name: &str, options: &ThreeMfOptions) -> KernelResult<Vec<u8>> {
    let buf = Cursor::new(Vec::new());
    let mut zip = ZipWriter::new(buf);
    let zip_opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("[Content_Types].xml", zip_opts)
        .map_err(|e| KernelError::Internal(format!("3MF ZIP error: {}", e)))?;
    zip.write_all(CONTENT_TYPES_XML.as_bytes())
        .map_err(|e| KernelError::Internal(format!("3MF write error: {}", e)))?;

    zip.start_file("_rels/.rels", zip_opts)
        .map_err(|e| KernelError::Internal(format!("3MF ZIP error: {}", e)))?;
    zip.write_all(RELS_XML.as_bytes())
        .map_err(|e| KernelError::Internal(format!("3MF write error: {}", e)))?;

    let model_xml = build_model_xml(mesh, model_name, options);
    zip.start_file("3D/3dmodel.model", zip_opts)
        .map_err(|e| KernelError::Internal(format!("3MF ZIP error: {}", e)))?;
    zip.write_all(model_xml.as_bytes())
        .map_err(|e| KernelError::Internal(format!("3MF write error: {}", e)))?;

    let cursor = zip
        .finish()
        .map_err(|e| KernelError::Internal(format!("3MF ZIP finalize error: {}", e)))?;

    Ok(cursor.into_inner())
}

fn build_model_xml(mesh: &TriMesh, model_name: &str, options: &ThreeMfOptions) -> String {
    let vc = mesh.vertex_count();
    let tc = mesh.triangle_count();
    let has_colors = options.vertex_colors && !mesh.colors.is_empty() && mesh.colors.len() == vc * 4;

    let mut xml = String::with_capacity(512 + 60 * vc + 80 * tc);

    // Header with configurable unit
    let _ = write!(xml, r#"<?xml version="1.0" encoding="UTF-8"?>
<model unit="{unit}" xml:lang="en-US" xmlns="http://schemas.microsoft.com/3dmanufacturing/core/2015/02""#,
        unit = options.unit);

    if has_colors {
        let _ = write!(xml, r#" xmlns:m="http://schemas.microsoft.com/3dmanufacturing/material/2015/02""#);
    }

    let _ = write!(xml, r#">
  <metadata name="Title">{}</metadata>
  <metadata name="Application">blockCAD</metadata>
  <resources>
"#, model_name);

    // Color group (if colors present)
    if has_colors {
        let _ = write!(xml, "    <m:colorgroup id=\"1\">\n");
        // Collect unique colors from per-vertex data, map to indices
        // For simplicity, write one color per vertex
        for i in 0..vc {
            let base = i * 4;
            let r = (mesh.colors[base] * 255.0) as u8;
            let g = (mesh.colors[base + 1] * 255.0) as u8;
            let b = (mesh.colors[base + 2] * 255.0) as u8;
            let a = (mesh.colors[base + 3] * 255.0) as u8;
            let _ = writeln!(xml, "      <m:color color=\"#{:02X}{:02X}{:02X}{:02X}\" />", r, g, b, a);
        }
        let _ = write!(xml, "    </m:colorgroup>\n");
    }

    let _ = write!(xml, r#"    <object id="2" type="model">
      <mesh>
        <vertices>
"#);

    for i in 0..vc {
        let base = i * 3;
        let _ = writeln!(xml, r#"          <vertex x="{:.6}" y="{:.6}" z="{:.6}" />"#,
            mesh.positions[base], mesh.positions[base + 1], mesh.positions[base + 2]);
    }

    let _ = write!(xml, "        </vertices>\n        <triangles>\n");

    for tri in mesh.indices.chunks(3) {
        if has_colors {
            // Reference color group and per-vertex color indices
            let _ = writeln!(xml,
                r#"          <triangle v1="{}" v2="{}" v3="{}" pid="1" p1="{}" p2="{}" p3="{}" />"#,
                tri[0], tri[1], tri[2], tri[0], tri[1], tri[2]);
        } else {
            let _ = writeln!(xml,
                r#"          <triangle v1="{}" v2="{}" v3="{}" />"#,
                tri[0], tri[1], tri[2]);
        }
    }

    let _ = write!(xml, r#"        </triangles>
      </mesh>
    </object>
  </resources>
  <build>
    <item objectid="2" />
  </build>
</model>
"#);

    xml
}

const CONTENT_TYPES_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml" />
  <Default Extension="model" ContentType="application/vnd.ms-package.3dmanufacturing-3dmodel+xml" />
</Types>"#;

const RELS_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Target="/3D/3dmodel.model" Id="rel0" Type="http://schemas.microsoft.com/3dmanufacturing/2013/01/3dmodel" />
</Relationships>"#;

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
    fn threemf_default_unit() {
        let bytes = export_3mf(&simple_triangle(), "test", &ThreeMfOptions::default()).unwrap();
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("3D/3dmodel.model").unwrap(), &mut xml).unwrap();
        assert!(xml.contains(r#"unit="millimeter""#));
    }

    #[test]
    fn threemf_inch_unit() {
        let opts = ThreeMfOptions { unit: "inch".into(), vertex_colors: false };
        let bytes = export_3mf(&simple_triangle(), "test", &opts).unwrap();
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("3D/3dmodel.model").unwrap(), &mut xml).unwrap();
        assert!(xml.contains(r#"unit="inch""#));
        assert!(!xml.contains("millimeter"));
    }

    #[test]
    fn threemf_vertex_colors() {
        let mesh = TriMesh {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            uvs: vec![], indices: vec![0, 1, 2], face_ids: vec![0],
            colors: vec![1.0, 0.0, 0.0, 1.0,  0.0, 1.0, 0.0, 1.0,  0.0, 0.0, 1.0, 1.0], // RGB vertices
        };
        let opts = ThreeMfOptions { unit: "millimeter".into(), vertex_colors: true };
        let bytes = export_3mf(&mesh, "test", &opts).unwrap();
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("3D/3dmodel.model").unwrap(), &mut xml).unwrap();
        assert!(xml.contains("<m:colorgroup"), "Should contain color group");
        assert!(xml.contains("<m:color"), "Should contain color entries");
        assert!(xml.contains(r#"pid="1""#), "Triangles should reference color group");
        assert!(xml.contains("#FF0000FF"), "Red vertex color");
        assert!(xml.contains("#00FF00FF"), "Green vertex color");
        assert!(xml.contains("#0000FFFF"), "Blue vertex color");
    }

    #[test]
    fn threemf_no_colors_when_disabled() {
        let mesh = TriMesh {
            positions: vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            normals: vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
            uvs: vec![], indices: vec![0, 1, 2], face_ids: vec![0],
            colors: vec![1.0, 0.0, 0.0, 1.0,  0.0, 1.0, 0.0, 1.0,  0.0, 0.0, 1.0, 1.0],
        };
        // vertex_colors disabled — should NOT emit colors even if mesh has them
        let opts = ThreeMfOptions { unit: "millimeter".into(), vertex_colors: false };
        let bytes = export_3mf(&mesh, "test", &opts).unwrap();
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut archive.by_name("3D/3dmodel.model").unwrap(), &mut xml).unwrap();
        assert!(!xml.contains("<m:colorgroup"), "Should NOT contain colors when disabled");
    }

    #[test]
    fn threemf_empty_mesh() {
        let bytes = export_3mf(&TriMesh::new(), "empty", &ThreeMfOptions::default()).unwrap();
        assert_eq!(bytes[0], b'P');
        assert_eq!(bytes[1], b'K');
    }
}
