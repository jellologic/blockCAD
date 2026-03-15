use crate::error::{KernelError, KernelResult};
use crate::geometry::Vec3;
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExtrudeParams {
    /// Direction of extrusion
    pub direction: Vec3,
    /// Depth of extrusion
    pub depth: f64,
    /// Whether to extrude symmetrically in both directions
    pub symmetric: bool,
    /// Draft angle in radians (for tapered extrusions)
    pub draft_angle: f64,
}

#[derive(Debug)]
pub struct ExtrudeOp;

impl Operation for ExtrudeOp {
    type Params = ExtrudeParams;

    fn execute(&self, params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        if params.depth <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "depth".into(),
                value: params.depth.to_string(),
            });
        }
        // TODO: Implement extrusion algorithm:
        // 1. Take sketch profile (closed loop of edges on input BRep)
        // 2. Sweep each edge along direction * depth → create side faces
        // 3. Create top cap face
        // 4. Stitch all faces into a shell → solid
        Err(KernelError::Operation {
            op: "extrude".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Extrude"
    }
}
