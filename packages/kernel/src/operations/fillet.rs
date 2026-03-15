use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FilletParams {
    pub edge_indices: Vec<u32>,
    pub radius: f64,
}

#[derive(Debug)]
pub struct FilletOp;

impl Operation for FilletOp {
    type Params = FilletParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "fillet".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Fillet"
    }
}
