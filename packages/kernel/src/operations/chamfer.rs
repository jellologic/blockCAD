use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ChamferParams {
    pub edge_indices: Vec<u32>,
    pub distance: f64,
    /// Optional second distance for asymmetric chamfer
    pub distance2: Option<f64>,
}

#[derive(Debug)]
pub struct ChamferOp;

impl Operation for ChamferOp {
    type Params = ChamferParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "chamfer".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Chamfer"
    }
}
