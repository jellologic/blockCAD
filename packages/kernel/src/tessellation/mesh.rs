use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};

fn quantize_vertex(x: f32, y: f32, z: f32) -> [i64; 3] {
    let scale = 1e5;
    [
        (x as f64 * scale).round() as i64,
        (y as f64 * scale).round() as i64,
        (z as f64 * scale).round() as i64,
    ]
}

/// Triangle mesh for visualization.
/// Uses f32 for GPU compatibility (computation stays f64 in the kernel).
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct TriMesh {
    /// Vertex positions: [x0, y0, z0, x1, y1, z1, ...]
    pub positions: Vec<f32>,
    /// Per-vertex normals, same layout as positions
    pub normals: Vec<f32>,
    /// Per-vertex UV coordinates: [u0, v0, u1, v1, ...]
    pub uvs: Vec<f32>,
    /// Triangle indices (u32 for WebGL/WebGPU compatibility)
    pub indices: Vec<u32>,
    /// Face ID that generated each triangle (for selection/highlight)
    pub face_ids: Vec<u32>,
    /// Optional per-vertex RGBA colors: [r0, g0, b0, a0, r1, ...] (0.0–1.0). Empty if unused.
    pub colors: Vec<f32>,
    /// Sharp/feature edge line segments: [x0,y0,z0, x1,y1,z1, ...] pairs of 3D positions
    pub edge_positions: Vec<f32>,
    /// Number of edge line segments
    pub edge_count: usize,
}

impl TriMesh {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of vertices
    pub fn vertex_count(&self) -> usize {
        self.positions.len() / 3
    }

    /// Number of triangles
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }

    /// Check if the mesh is watertight (every directed edge has a matching reverse edge).
    pub fn is_watertight(&self) -> bool {
        if self.indices.is_empty() {
            return true; // empty mesh is trivially watertight
        }

        // Map each vertex index to its quantized position
        let mut vertex_key: HashMap<u32, [i64; 3]> = HashMap::new();
        for i in 0..self.vertex_count() {
            let x = self.positions[i * 3];
            let y = self.positions[i * 3 + 1];
            let z = self.positions[i * 3 + 2];
            vertex_key.insert(i as u32, quantize_vertex(x, y, z));
        }

        // Count directed edges by quantized position
        let mut edge_count: HashMap<([i64; 3], [i64; 3]), i32> = HashMap::new();
        for tri in self.indices.chunks(3) {
            let keys: Vec<[i64; 3]> = tri.iter().map(|&idx| vertex_key[&idx]).collect();
            for e in 0..3 {
                let a = keys[e];
                let b = keys[(e + 1) % 3];
                *edge_count.entry((a, b)).or_insert(0) += 1;
            }
        }

        // Every directed edge (a, b) must have exactly one matching (b, a)
        for (&(a, b), &count) in &edge_count {
            let reverse = edge_count.get(&(b, a)).copied().unwrap_or(0);
            if count != reverse {
                return false;
            }
        }

        true
    }

    /// Fix triangle winding to match per-vertex normals.
    /// For each triangle, if the cross-product normal disagrees with the
    /// vertex normal, swap two indices to flip the winding.
    pub fn fix_winding(&mut self) {
        for tri in self.indices.chunks_mut(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v0 = [self.positions[i0*3], self.positions[i0*3+1], self.positions[i0*3+2]];
            let v1 = [self.positions[i1*3], self.positions[i1*3+1], self.positions[i1*3+2]];
            let v2 = [self.positions[i2*3], self.positions[i2*3+1], self.positions[i2*3+2]];

            let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
            let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
            let cross = [
                e1[1]*e2[2] - e1[2]*e2[1],
                e1[2]*e2[0] - e1[0]*e2[2],
                e1[0]*e2[1] - e1[1]*e2[0],
            ];

            let vn = [self.normals[i0*3], self.normals[i0*3+1], self.normals[i0*3+2]];
            let dot = cross[0]*vn[0] + cross[1]*vn[1] + cross[2]*vn[2];

            if dot < 0.0 {
                tri.swap(1, 2);
            }
        }
    }

    /// Merge another mesh into this one, offsetting indices
    pub fn merge(&mut self, other: &TriMesh) {
        let offset = self.vertex_count() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.uvs.extend_from_slice(&other.uvs);
        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
        self.face_ids.extend_from_slice(&other.face_ids);
        self.colors.extend_from_slice(&other.colors);
        self.edge_positions.extend_from_slice(&other.edge_positions);
        self.edge_count += other.edge_count;
    }

    /// Compute sharp/feature edges based on the dihedral angle between adjacent triangles.
    /// Edges where the angle between face normals exceeds `threshold_degrees` are included,
    /// as well as boundary edges (edges with only one adjacent triangle).
    pub fn compute_feature_edges(&mut self, threshold_degrees: f32) {
        let threshold_cos = (threshold_degrees * std::f32::consts::PI / 180.0).cos();

        // Build edge adjacency: map quantized edge (v0, v1) -> list of triangle indices
        // We use quantized positions so coincident vertices are merged.
        let tri_count = self.triangle_count();
        let mut edge_to_tris: HashMap<([i64; 3], [i64; 3]), Vec<usize>> = HashMap::new();

        // Precompute quantized positions for each vertex
        let vc = self.vertex_count();
        let mut qv: Vec<[i64; 3]> = Vec::with_capacity(vc);
        for i in 0..vc {
            qv.push(quantize_vertex(
                self.positions[i * 3],
                self.positions[i * 3 + 1],
                self.positions[i * 3 + 2],
            ));
        }

        // Precompute face normals for each triangle
        let mut face_normals: Vec<[f32; 3]> = Vec::with_capacity(tri_count);
        for tri in self.indices.chunks(3) {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;

            let v0 = [self.positions[i0*3], self.positions[i0*3+1], self.positions[i0*3+2]];
            let v1 = [self.positions[i1*3], self.positions[i1*3+1], self.positions[i1*3+2]];
            let v2 = [self.positions[i2*3], self.positions[i2*3+1], self.positions[i2*3+2]];

            let e1 = [v1[0]-v0[0], v1[1]-v0[1], v1[2]-v0[2]];
            let e2 = [v2[0]-v0[0], v2[1]-v0[1], v2[2]-v0[2]];
            let cross = [
                e1[1]*e2[2] - e1[2]*e2[1],
                e1[2]*e2[0] - e1[0]*e2[2],
                e1[0]*e2[1] - e1[1]*e2[0],
            ];
            let len = (cross[0]*cross[0] + cross[1]*cross[1] + cross[2]*cross[2]).sqrt();
            if len > 1e-12 {
                face_normals.push([cross[0]/len, cross[1]/len, cross[2]/len]);
            } else {
                face_normals.push([0.0, 0.0, 0.0]);
            }
        }

        // Build adjacency map using canonical (sorted) quantized edge keys
        for (tri_idx, tri) in self.indices.chunks(3).enumerate() {
            for e in 0..3 {
                let a = qv[tri[e] as usize];
                let b = qv[tri[(e + 1) % 3] as usize];
                // Canonical key: smaller vertex first
                let key = if a < b { (a, b) } else { (b, a) };
                edge_to_tris.entry(key).or_default().push(tri_idx);
            }
        }

        // Track which edges we've already added (by canonical quantized key)
        let mut added_edges: std::collections::HashSet<([i64; 3], [i64; 3])> = std::collections::HashSet::new();

        self.edge_positions.clear();
        self.edge_count = 0;

        for (&(qa, qb), tris) in &edge_to_tris {
            if !added_edges.insert((qa, qb)) {
                continue; // Already processed
            }

            let is_feature = if tris.len() == 1 {
                // Boundary edge - always a feature edge
                true
            } else if tris.len() == 2 {
                // Check angle between the two face normals
                let n0 = face_normals[tris[0]];
                let n1 = face_normals[tris[1]];
                let dot = n0[0]*n1[0] + n0[1]*n1[1] + n0[2]*n1[2];
                dot < threshold_cos
            } else {
                // Non-manifold edge (more than 2 triangles) - always a feature
                true
            };

            if is_feature {
                // Find actual vertex positions for this edge.
                // Use the first triangle's actual vertex data.
                let tri = &self.indices[tris[0] * 3..tris[0] * 3 + 3];
                let mut pa = None;
                let mut pb = None;
                for e in 0..3 {
                    let vi = tri[e] as usize;
                    let vj = tri[(e + 1) % 3] as usize;
                    let qi = qv[vi];
                    let qj = qv[vj];
                    let key = if qi < qj { (qi, qj) } else { (qj, qi) };
                    if key == (qa, qb) {
                        pa = Some(vi);
                        pb = Some(vj);
                        break;
                    }
                }
                if let (Some(a), Some(b)) = (pa, pb) {
                    self.edge_positions.push(self.positions[a * 3]);
                    self.edge_positions.push(self.positions[a * 3 + 1]);
                    self.edge_positions.push(self.positions[a * 3 + 2]);
                    self.edge_positions.push(self.positions[b * 3]);
                    self.edge_positions.push(self.positions[b * 3 + 1]);
                    self.edge_positions.push(self.positions[b * 3 + 2]);
                    self.edge_count += 1;
                }
            }
        }
    }

    /// Validate mesh integrity
    pub fn validate(&self) -> KernelResult<()> {
        // Positions must be a multiple of 3
        if self.positions.len() % 3 != 0 {
            return Err(KernelError::Internal(
                "Position array length not a multiple of 3".into(),
            ));
        }
        // Normals must match positions
        if self.normals.len() != self.positions.len() {
            return Err(KernelError::Internal(
                "Normal array length does not match positions".into(),
            ));
        }
        // Indices must be a multiple of 3
        if self.indices.len() % 3 != 0 {
            return Err(KernelError::Internal(
                "Index array length not a multiple of 3".into(),
            ));
        }
        // All indices must be in bounds
        let vc = self.vertex_count() as u32;
        for &idx in &self.indices {
            if idx >= vc {
                return Err(KernelError::Internal(format!(
                    "Index {} out of bounds (vertex count = {})",
                    idx, vc
                )));
            }
        }
        // Colors must be empty or match vertex count × 4
        if !self.colors.is_empty() && self.colors.len() != self.vertex_count() * 4 {
            return Err(KernelError::Internal(format!(
                "Color array length {} does not match vertex count × 4 = {}",
                self.colors.len(),
                self.vertex_count() * 4
            )));
        }
        // Check for degenerate triangles
        for tri in self.indices.chunks(3) {
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                return Err(KernelError::Internal(format!(
                    "Degenerate triangle: [{}, {}, {}]",
                    tri[0], tri[1], tri[2]
                )));
            }
        }
        // Check watertightness
        if !self.is_watertight() {
            return Err(KernelError::Topology("Mesh is not watertight".into()));
        }
        Ok(())
    }

    /// Convert to a flat byte buffer for zero-copy WASM transfer.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Write vertex count
        let vc = self.vertex_count() as u32;
        buf.extend_from_slice(&vc.to_le_bytes());

        // Write positions
        for &v in &self.positions {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        // Write normals
        for &v in &self.normals {
            buf.extend_from_slice(&v.to_le_bytes());
        }
        // Write UVs
        for &v in &self.uvs {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        // Write triangle count
        let tc = self.triangle_count() as u32;
        buf.extend_from_slice(&tc.to_le_bytes());

        // Write indices
        for &i in &self.indices {
            buf.extend_from_slice(&i.to_le_bytes());
        }

        // Write face IDs (one per triangle)
        for &id in &self.face_ids {
            buf.extend_from_slice(&id.to_le_bytes());
        }

        // Write edge count
        let ec = self.edge_count as u32;
        buf.extend_from_slice(&ec.to_le_bytes());

        // Write edge positions (6 floats per edge: 2 points × 3 coords)
        for &v in &self.edge_positions {
            buf.extend_from_slice(&v.to_le_bytes());
        }

        buf
    }
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
    fn single_triangle_not_watertight() {
        let mesh = simple_triangle();
        // A single triangle is not watertight (open surface)
        assert!(mesh.validate().is_err());
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn degenerate_triangle_fails() {
        let mesh = TriMesh {
            positions: vec![0.0; 9],
            normals: vec![0.0; 9],
            uvs: vec![0.0; 6],
            indices: vec![0, 0, 1], // degenerate
            face_ids: vec![0],
            colors: vec![],
            ..Default::default()
        };
        assert!(mesh.validate().is_err());
    }

    #[test]
    fn merge_meshes() {
        let mut a = simple_triangle();
        let b = simple_triangle();
        a.merge(&b);
        assert_eq!(a.vertex_count(), 6);
        assert_eq!(a.triangle_count(), 2);
        // Second triangle should have offset indices
        assert_eq!(a.indices[3], 3);
        assert_eq!(a.indices[4], 4);
        assert_eq!(a.indices[5], 5);
    }

    #[test]
    fn to_bytes_roundtrip_sanity() {
        let mesh = simple_triangle();
        let bytes = mesh.to_bytes();
        // First 4 bytes = vertex count (3)
        let vc = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(vc, 3);
    }

    /// Build a simple box mesh (axis-aligned unit cube) for edge testing.
    /// 6 faces, 12 triangles, 8 unique corner positions (but 24 vertices for per-face normals).
    fn box_mesh() -> TriMesh {
        // 8 corners of a unit cube
        let corners: [[f32; 3]; 8] = [
            [0.0, 0.0, 0.0], // 0
            [1.0, 0.0, 0.0], // 1
            [1.0, 1.0, 0.0], // 2
            [0.0, 1.0, 0.0], // 3
            [0.0, 0.0, 1.0], // 4
            [1.0, 0.0, 1.0], // 5
            [1.0, 1.0, 1.0], // 6
            [0.0, 1.0, 1.0], // 7
        ];
        let face_normals: [[f32; 3]; 6] = [
            [ 0.0,  0.0, -1.0], // front  (z=0): 0,1,2,3
            [ 0.0,  0.0,  1.0], // back   (z=1): 4,5,6,7
            [ 0.0, -1.0,  0.0], // bottom (y=0): 0,1,5,4
            [ 0.0,  1.0,  0.0], // top    (y=1): 3,2,6,7
            [-1.0,  0.0,  0.0], // left   (x=0): 0,3,7,4
            [ 1.0,  0.0,  0.0], // right  (x=1): 1,2,6,5
        ];
        // Each face: 4 corner indices into `corners`, forming a quad split into 2 triangles
        let face_corners: [[usize; 4]; 6] = [
            [0, 1, 2, 3], // front
            [5, 4, 7, 6], // back
            [0, 4, 5, 1], // bottom
            [3, 2, 6, 7], // top
            [0, 3, 7, 4], // left
            [1, 5, 6, 2], // right
        ];

        let mut mesh = TriMesh::new();
        for (fi, fc) in face_corners.iter().enumerate() {
            let base = mesh.vertex_count() as u32;
            for &ci in fc {
                mesh.positions.extend_from_slice(&corners[ci]);
                mesh.normals.extend_from_slice(&face_normals[fi]);
                mesh.uvs.extend_from_slice(&[0.0, 0.0]);
            }
            // Two triangles per face
            mesh.indices.extend_from_slice(&[base, base+1, base+2]);
            mesh.indices.extend_from_slice(&[base, base+2, base+3]);
            mesh.face_ids.push(fi as u32);
            mesh.face_ids.push(fi as u32);
        }
        mesh
    }

    #[test]
    fn box_has_12_feature_edges() {
        let mut mesh = box_mesh();
        assert!(mesh.is_watertight());
        mesh.compute_feature_edges(15.0);
        // A box has 12 edges, all at 90° (well above 15° threshold)
        assert_eq!(mesh.edge_count, 12, "Box should have 12 feature edges, got {}", mesh.edge_count);
        assert_eq!(mesh.edge_positions.len(), 12 * 6);
    }

    #[test]
    fn coplanar_faces_no_feature_edges() {
        // Two coplanar triangles sharing an edge - should produce 0 feature edges
        // (only boundary edges, which are 4 outer edges)
        let mut mesh = TriMesh {
            positions: vec![
                0.0, 0.0, 0.0,  // 0
                1.0, 0.0, 0.0,  // 1
                1.0, 1.0, 0.0,  // 2
                0.0, 0.0, 0.0,  // 3 (=0)
                1.0, 1.0, 0.0,  // 4 (=2)
                0.0, 1.0, 0.0,  // 5
            ],
            normals: vec![
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ],
            uvs: vec![0.0; 12],
            indices: vec![0, 1, 2, 3, 4, 5],
            face_ids: vec![0, 0],
            colors: vec![],
            ..Default::default()
        };
        mesh.compute_feature_edges(15.0);
        // The shared internal edge has 0° dihedral angle - not a feature edge
        // Boundary edges (4 of them) have only 1 adjacent triangle - they ARE feature edges
        assert_eq!(mesh.edge_count, 4, "Coplanar quad should have 4 boundary feature edges, got {}", mesh.edge_count);
    }

    #[test]
    fn feature_edges_in_to_bytes() {
        let mut mesh = box_mesh();
        mesh.compute_feature_edges(15.0);
        let bytes = mesh.to_bytes();

        // Parse back: skip to edge data
        let vc = mesh.vertex_count();
        let tc = mesh.triangle_count();
        let offset = 4 // vertex_count
            + vc * 3 * 4 // positions
            + vc * 3 * 4 // normals
            + vc * 2 * 4 // uvs
            + 4 // triangle_count
            + tc * 3 * 4 // indices
            + tc * 4; // face_ids
        let ec = u32::from_le_bytes([
            bytes[offset], bytes[offset+1], bytes[offset+2], bytes[offset+3]
        ]);
        assert_eq!(ec, 12, "Serialized edge count should be 12");
        // Check total expected size
        let expected_size = offset + 4 + 12 * 6 * 4;
        assert_eq!(bytes.len(), expected_size, "Byte buffer size mismatch");
    }

    #[test]
    fn merge_preserves_edge_data() {
        let mut a = box_mesh();
        a.compute_feature_edges(15.0);
        let mut b = box_mesh();
        b.compute_feature_edges(15.0);
        let a_edges = a.edge_count;
        let b_edges = b.edge_count;
        a.merge(&b);
        assert_eq!(a.edge_count, a_edges + b_edges);
        assert_eq!(a.edge_positions.len(), (a_edges + b_edges) * 6);
    }
}
