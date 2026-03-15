use wasm_bindgen::prelude::*;

use crate::tessellation::TriMesh;

/// WASM handle for mesh data, providing typed array access to vertex/index buffers.
#[wasm_bindgen]
pub struct MeshHandle {
    mesh: TriMesh,
}

#[wasm_bindgen]
impl MeshHandle {
    pub fn vertex_count(&self) -> usize {
        self.mesh.vertex_count()
    }

    pub fn triangle_count(&self) -> usize {
        self.mesh.triangle_count()
    }

    /// Get the mesh as a flat byte buffer for zero-copy JS typed array access.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.mesh.to_bytes()
    }
}

impl MeshHandle {
    pub fn from_mesh(mesh: TriMesh) -> Self {
        Self { mesh }
    }
}
