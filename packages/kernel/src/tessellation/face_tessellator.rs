use crate::error::{KernelError, KernelResult};

use super::mesh::TriMesh;
use super::params::TessellationParams;

/// Tessellate a single face into triangles.
///
/// Algorithm (to be implemented):
/// 1. Sample the surface within the face's parametric domain
/// 2. Respect the face's boundary loops (outer + inner)
/// 3. Use constrained Delaunay triangulation in parameter space
/// 4. Refine based on chord_tolerance and angle_tolerance
pub fn tessellate_face(
    _face_id: u32,
    _params: &TessellationParams,
) -> KernelResult<TriMesh> {
    Err(KernelError::Internal(
        "Face tessellator not yet implemented".into(),
    ))
}
