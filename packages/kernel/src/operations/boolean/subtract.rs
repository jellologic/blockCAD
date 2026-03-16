use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubtractParams {}

#[derive(Debug)]
pub struct SubtractOp;

impl Operation for SubtractOp {
    type Params = SubtractParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "boolean_subtract".into(),
            detail: "Use csg_subtract() directly with both BReps".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Boolean Subtract"
    }
}
