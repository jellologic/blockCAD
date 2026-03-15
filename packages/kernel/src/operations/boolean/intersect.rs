use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IntersectParams {}

#[derive(Debug)]
pub struct IntersectOp;

impl Operation for IntersectOp {
    type Params = IntersectParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "boolean_intersect".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Boolean Intersect"
    }
}
