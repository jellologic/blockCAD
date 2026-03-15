use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShellParams {
    /// Face indices to remove (creating openings)
    pub faces_to_remove: Vec<u32>,
    /// Wall thickness
    pub thickness: f64,
}

#[derive(Debug)]
pub struct ShellOp;

impl Operation for ShellOp {
    type Params = ShellParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "shell".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Shell"
    }
}
