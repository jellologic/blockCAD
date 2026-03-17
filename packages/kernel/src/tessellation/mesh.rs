use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};

#[inline]
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
    /// Uses bytemuck for bulk slice casting instead of per-element serialization.
    pub fn to_bytes(&self) -> Vec<u8> {
        let vc = self.vertex_count() as u32;
        let tc = self.triangle_count() as u32;

        // Pre-compute total size to avoid reallocations
        let total_size = 4 // vertex count
            + self.positions.len() * 4
            + self.normals.len() * 4
            + self.uvs.len() * 4
            + 4 // triangle count
            + self.indices.len() * 4
            + self.face_ids.len() * 4;
        let mut buf = Vec::with_capacity(total_size);

        // Write vertex count
        buf.extend_from_slice(&vc.to_le_bytes());

        // Write positions, normals, UVs as bulk slices
        buf.extend_from_slice(bytemuck::cast_slice::<f32, u8>(&self.positions));
        buf.extend_from_slice(bytemuck::cast_slice::<f32, u8>(&self.normals));
        buf.extend_from_slice(bytemuck::cast_slice::<f32, u8>(&self.uvs));

        // Write triangle count
        buf.extend_from_slice(&tc.to_le_bytes());

        // Write indices and face IDs as bulk slices
        buf.extend_from_slice(bytemuck::cast_slice::<u32, u8>(&self.indices));
        buf.extend_from_slice(bytemuck::cast_slice::<u32, u8>(&self.face_ids));

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
}
