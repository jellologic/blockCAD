use crate::error::{KernelError, KernelResult};
use crate::geometry::Vec3;
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinearPatternParams {
    pub direction: Vec3,
    pub spacing: f64,
    pub count: u32,
    /// Optional second direction for 2D patterns
    pub direction2: Option<Vec3>,
    pub spacing2: Option<f64>,
    pub count2: Option<u32>,
}

#[derive(Debug)]
pub struct LinearPatternOp;

impl Operation for LinearPatternOp {
    type Params = LinearPatternParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "linear_pattern".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Linear Pattern"
    }
}
