use crate::error::{KernelError, KernelResult};
use crate::geometry::Pt3;

use super::params::TessellationParams;

/// Tessellate a curve into a polyline (sequence of points).
///
/// Uses adaptive subdivision based on chord tolerance:
/// recursively subdivide segments where the midpoint deviation
/// from the chord exceeds the tolerance.
pub fn tessellate_edge(
    _params: &TessellationParams,
) -> KernelResult<Vec<Pt3>> {
    Err(KernelError::Internal(
        "Edge tessellator not yet implemented".into(),
    ))
}
