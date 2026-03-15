use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MirrorParams {
    pub plane_origin: Pt3,
    pub plane_normal: Vec3,
}

#[derive(Debug)]
pub struct MirrorOp;

impl Operation for MirrorOp {
    type Params = MirrorParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "mirror".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Mirror"
    }
}
