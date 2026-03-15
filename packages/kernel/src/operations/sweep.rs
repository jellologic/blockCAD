use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SweepParams {
    /// Index of the path curve in the BRep's curve storage
    pub path_curve_index: usize,
    /// Twist angle along the sweep (radians)
    pub twist: f64,
}

#[derive(Debug)]
pub struct SweepOp;

impl Operation for SweepOp {
    type Params = SweepParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "sweep".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Sweep"
    }
}
