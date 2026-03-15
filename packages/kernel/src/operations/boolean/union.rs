use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnionParams {
    // The tool body will be provided via a separate mechanism (multi-body input)
}

#[derive(Debug)]
pub struct UnionOp;

impl Operation for UnionOp {
    type Params = UnionParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "boolean_union".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Boolean Union"
    }
}
