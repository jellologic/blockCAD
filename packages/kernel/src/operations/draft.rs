use crate::error::{KernelError, KernelResult};
use crate::geometry::Vec3;
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DraftParams {
    pub face_indices: Vec<u32>,
    pub pull_direction: Vec3,
    /// Draft angle in radians
    pub angle: f64,
}

#[derive(Debug)]
pub struct DraftOp;

impl Operation for DraftOp {
    type Params = DraftParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "draft".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Draft"
    }
}
