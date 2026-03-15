use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LoftParams {
    /// Indices of profile loops in the feature tree
    pub profile_indices: Vec<usize>,
    /// Whether the loft is closed (last profile connects to first)
    pub closed: bool,
}

#[derive(Debug)]
pub struct LoftOp;

impl Operation for LoftOp {
    type Params = LoftParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "loft".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Loft"
    }
}
