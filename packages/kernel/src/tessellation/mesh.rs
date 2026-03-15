use crate::error::{KernelError, KernelResult};

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

    /// Merge another mesh into this one, offsetting indices
    pub fn merge(&mut self, other: &TriMesh) {
        let offset = self.vertex_count() as u32;
        self.positions.extend_from_slice(&other.positions);
        self.normals.extend_from_slice(&other.normals);
        self.uvs.extend_from_slice(&other.uvs);
        self.indices
            .extend(other.indices.iter().map(|i| i + offset));
        self.face_ids.extend_from_slice(&other.face_ids);
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
        // Check for degenerate triangles
        for tri in self.indices.chunks(3) {
            if tri[0] == tri[1] || tri[1] == tri[2] || tri[0] == tri[2] {
                return Err(KernelError::Internal(format!(
                    "Degenerate triangle: [{}, {}, {}]",
                    tri[0], tri[1], tri[2]
                )));
            }
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
        }
    }

    #[test]
    fn valid_triangle() {
        let mesh = simple_triangle();
        assert!(mesh.validate().is_ok());
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
