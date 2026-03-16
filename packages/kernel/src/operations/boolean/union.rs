use crate::error::KernelResult;
use crate::topology::BRep;
use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnionParams {
    // Tool BRep is provided separately via the evaluator
}

#[derive(Debug)]
pub struct UnionOp;

impl Operation for UnionOp {
    type Params = UnionParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        // Actual CSG is done in the evaluator which has both BReps
        Err(crate::error::KernelError::Operation {
            op: "boolean_union".into(),
            detail: "Use csg_union() directly with both BReps".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Boolean Union"
    }
}
