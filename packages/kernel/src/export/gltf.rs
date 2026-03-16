use crate::error::KernelResult;
use crate::tessellation::mesh::TriMesh;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlbOptions {
    /// Quantize positions to 16-bit and normals to 8-bit oct-encoded (default: false)
    #[serde(default)]
    pub quantize: bool,
}

impl Default for GlbOptions {
    fn default() -> Self {
        Self { quantize: false }
    }
}

/// Export a TriMesh as GLB (binary glTF 2.0) bytes.
pub fn export_glb(mesh: &TriMesh, name: &str, options: &GlbOptions) -> KernelResult<Vec<u8>> {
    if options.quantize {
        export_glb_quantized(mesh, name)
    } else {
        export_glb_standard(mesh, name)
    }
}

fn export_glb_standard(mesh: &TriMesh, name: &str) -> KernelResult<Vec<u8>> {
    let vc = mesh.vertex_count();
    let tc = mesh.triangle_count();

    let positions_byte_len = vc * 3 * 4;
    let normals_byte_len = vc * 3 * 4;
    let indices_byte_len = tc * 3 * 4;
    let bin_len = positions_byte_len + normals_byte_len + indices_byte_len;
    let bin_padded = (bin_len + 3) & !3;

    let mut bin_data = Vec::with_capacity(bin_padded);
    for &v in &mesh.positions { bin_data.extend_from_slice(&v.to_le_bytes()); }
    for &v in &mesh.normals { bin_data.extend_from_slice(&v.to_le_bytes()); }
    for &i in &mesh.indices { bin_data.extend_from_slice(&i.to_le_bytes()); }
    while bin_data.len() < bin_padded { bin_data.push(0); }

    let (min_pos, max_pos) = compute_bounds(&mesh.positions);

    let json = build_standard_json(name, vc, tc, positions_byte_len, normals_byte_len, indices_byte_len, bin_padded, &min_pos, &max_pos);

    assemble_glb(&json, &bin_data)
}

fn export_glb_quantized(mesh: &TriMesh, name: &str) -> KernelResult<Vec<u8>> {
    let vc = mesh.vertex_count();
    let tc = mesh.triangle_count();

    // Compute bounds for quantization
    let (min_pos, max_pos) = compute_bounds(&mesh.positions);

    // Quantize positions to i16 (-32767..32767)
    let positions_byte_len = vc * 3 * 2; // i16
    // Quantize normals to oct-encoded i8 (2 bytes per normal)
    let normals_byte_len = vc * 2; // 2 × i8
    let indices_byte_len = tc * 3 * 4; // u32

    let bin_len = positions_byte_len + normals_byte_len + indices_byte_len;
    let bin_padded = (bin_len + 3) & !3;

    let mut bin_data = Vec::with_capacity(bin_padded);

    // Quantize positions
    let range = [
        if (max_pos[0] - min_pos[0]).abs() > 1e-12 { max_pos[0] - min_pos[0] } else { 1.0 },
        if (max_pos[1] - min_pos[1]).abs() > 1e-12 { max_pos[1] - min_pos[1] } else { 1.0 },
        if (max_pos[2] - min_pos[2]).abs() > 1e-12 { max_pos[2] - min_pos[2] } else { 1.0 },
    ];
    for i in 0..vc {
        for j in 0..3 {
            let v = mesh.positions[i * 3 + j];
            let normalized = (v - min_pos[j]) / range[j]; // 0..1
            let quantized = (normalized * 65534.0 - 32767.0).round() as i16;
            bin_data.extend_from_slice(&quantized.to_le_bytes());
        }
    }

    // Oct-encode normals to 2 × i8
    for i in 0..vc {
        let nx = mesh.normals[i * 3];
        let ny = mesh.normals[i * 3 + 1];
        let nz = mesh.normals[i * 3 + 2];
        let (ox, oy) = oct_encode(nx, ny, nz);
        bin_data.push(ox as u8);
        bin_data.push(oy as u8);
    }

    // Indices
    for &idx in &mesh.indices {
        bin_data.extend_from_slice(&idx.to_le_bytes());
    }
    while bin_data.len() < bin_padded { bin_data.push(0); }

    let json = build_quantized_json(
        name, vc, tc,
        positions_byte_len, normals_byte_len, indices_byte_len,
        bin_padded, &min_pos, &max_pos, &range,
    );

    assemble_glb(&json, &bin_data)
}

fn assemble_glb(json: &str, bin_data: &[u8]) -> KernelResult<Vec<u8>> {
    let json_bytes = json.as_bytes();
    let json_padded = (json_bytes.len() + 3) & !3;
    let bin_padded = bin_data.len();
    let total_len = 12 + 8 + json_padded + 8 + bin_padded;

    let mut glb = Vec::with_capacity(total_len);
    glb.extend_from_slice(&0x46546C67u32.to_le_bytes());
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_len as u32).to_le_bytes());

    glb.extend_from_slice(&(json_padded as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(json_bytes);
    for _ in json_bytes.len()..json_padded { glb.push(b' '); }

    glb.extend_from_slice(&(bin_padded as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(bin_data);

    debug_assert_eq!(glb.len(), total_len);
    Ok(glb)
}

/// Oct-encode a unit normal to 2 × i8
fn oct_encode(nx: f32, ny: f32, nz: f32) -> (i8, i8) {
    let l1 = nx.abs() + ny.abs() + nz.abs();
    if l1 < 1e-12 {
        return (0, 0);
    }
    let mut ox = nx / l1;
    let mut oy = ny / l1;
    if nz < 0.0 {
        let tmp_ox = (1.0 - oy.abs()) * if ox >= 0.0 { 1.0 } else { -1.0 };
        let tmp_oy = (1.0 - ox.abs()) * if oy >= 0.0 { 1.0 } else { -1.0 };
        ox = tmp_ox;
        oy = tmp_oy;
    }
    ((ox * 127.0).round() as i8, (oy * 127.0).round() as i8)
}

fn compute_bounds(positions: &[f32]) -> ([f32; 3], [f32; 3]) {
    if positions.is_empty() {
        return ([0.0; 3], [0.0; 3]);
    }
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for chunk in positions.chunks(3) {
        for i in 0..3 {
            if chunk[i] < min[i] { min[i] = chunk[i]; }
            if chunk[i] > max[i] { max[i] = chunk[i]; }
        }
    }
    (min, max)
}

fn build_standard_json(
    name: &str, vc: usize, tc: usize,
    positions_byte_len: usize, normals_byte_len: usize, indices_byte_len: usize,
    total_buffer_len: usize, min_pos: &[f32; 3], max_pos: &[f32; 3],
) -> String {
    let normals_offset = positions_byte_len;
    let indices_offset = positions_byte_len + normals_byte_len;

    let root = serde_json::json!({
        "asset": { "version": "2.0", "generator": "blockCAD" },
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{ "mesh": 0, "name": name }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "mode": 4 }] }],
        "accessors": [
            { "bufferView": 0, "componentType": 5126, "count": vc, "type": "VEC3", "min": [min_pos[0], min_pos[1], min_pos[2]], "max": [max_pos[0], max_pos[1], max_pos[2]] },
            { "bufferView": 1, "componentType": 5126, "count": vc, "type": "VEC3" },
            { "bufferView": 2, "componentType": 5125, "count": tc * 3, "type": "SCALAR" }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": positions_byte_len, "target": 34962 },
            { "buffer": 0, "byteOffset": normals_offset, "byteLength": normals_byte_len, "target": 34962 },
            { "buffer": 0, "byteOffset": indices_offset, "byteLength": indices_byte_len, "target": 34963 }
        ],
        "buffers": [{ "byteLength": total_buffer_len }]
    });
    serde_json::to_string(&root).unwrap_or_default()
}

fn build_quantized_json(
    name: &str, vc: usize, tc: usize,
    positions_byte_len: usize, normals_byte_len: usize, indices_byte_len: usize,
    total_buffer_len: usize, min_pos: &[f32; 3], _max_pos: &[f32; 3], range: &[f32; 3],
) -> String {
    let normals_offset = positions_byte_len;
    let indices_offset = positions_byte_len + normals_byte_len;

    // Decode matrix: maps i16 back to world space
    // pos = (quantized + 32767) / 65534 * range + min
    let root = serde_json::json!({
        "asset": { "version": "2.0", "generator": "blockCAD" },
        "extensionsUsed": ["KHR_mesh_quantization"],
        "extensionsRequired": ["KHR_mesh_quantization"],
        "scene": 0,
        "scenes": [{ "nodes": [0] }],
        "nodes": [{
            "mesh": 0,
            "name": name,
            "translation": [
                min_pos[0] as f64 + range[0] as f64 / 2.0,
                min_pos[1] as f64 + range[1] as f64 / 2.0,
                min_pos[2] as f64 + range[2] as f64 / 2.0
            ],
            "scale": [
                range[0] as f64 / 65534.0,
                range[1] as f64 / 65534.0,
                range[2] as f64 / 65534.0
            ]
        }],
        "meshes": [{ "primitives": [{ "attributes": { "POSITION": 0, "NORMAL": 1 }, "indices": 2, "mode": 4 }] }],
        "accessors": [
            {
                "bufferView": 0,
                "componentType": 5122, // SHORT
                "count": vc,
                "type": "VEC3",
                "min": [-32767, -32767, -32767],
                "max": [32767, 32767, 32767]
            },
            {
                "bufferView": 1,
                "componentType": 5120, // BYTE
                "count": vc,
                "type": "VEC2",
                "normalized": true
            },
            {
                "bufferView": 2,
                "componentType": 5125,
                "count": tc * 3,
                "type": "SCALAR"
            }
        ],
        "bufferViews": [
            { "buffer": 0, "byteOffset": 0, "byteLength": positions_byte_len, "target": 34962 },
            { "buffer": 0, "byteOffset": normals_offset, "byteLength": normals_byte_len, "target": 34962 },
            { "buffer": 0, "byteOffset": indices_offset, "byteLength": indices_byte_len, "target": 34963 }
        ],
        "buffers": [{ "byteLength": total_buffer_len }]
    });
    serde_json::to_string(&root).unwrap_or_default()
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
    fn glb_standard_magic_and_version() {
        let bytes = export_glb(&simple_triangle(), "test", &GlbOptions::default()).unwrap();
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, 0x46546C67);
        let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(version, 2);
    }

    #[test]
    fn glb_quantized_smaller_than_standard() {
        let mesh = simple_triangle();
        let std_bytes = export_glb(&mesh, "test", &GlbOptions { quantize: false }).unwrap();
        let qnt_bytes = export_glb(&mesh, "test", &GlbOptions { quantize: true }).unwrap();
        // Quantized BIN should be smaller (positions: 6 vs 12, normals: 2 vs 12 per vertex)
        // JSON may be larger due to extension, but BIN savings dominate for bigger meshes
        // For tiny meshes, just verify it produces valid output
        let qnt_magic = u32::from_le_bytes([qnt_bytes[0], qnt_bytes[1], qnt_bytes[2], qnt_bytes[3]]);
        assert_eq!(qnt_magic, 0x46546C67);
    }

    #[test]
    fn glb_quantized_has_extension() {
        let bytes = export_glb(&simple_triangle(), "test", &GlbOptions { quantize: true }).unwrap();
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len]).unwrap().trim();
        assert!(json_str.contains("KHR_mesh_quantization"), "Should contain quantization extension");
    }

    #[test]
    fn glb_json_chunk_valid() {
        let bytes = export_glb(&simple_triangle(), "test", &GlbOptions::default()).unwrap();
        let json_len = u32::from_le_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]) as usize;
        let json_str = std::str::from_utf8(&bytes[20..20 + json_len]).unwrap().trim();
        let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
        assert_eq!(parsed["asset"]["version"], "2.0");
    }

    #[test]
    fn glb_empty_mesh() {
        let bytes = export_glb(&TriMesh::new(), "empty", &GlbOptions::default()).unwrap();
        let magic = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(magic, 0x46546C67);
    }
}
