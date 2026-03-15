use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircularPatternParams {
    pub axis_origin: Pt3,
    pub axis_direction: Vec3,
    pub count: u32,
    /// Total angle to distribute instances over (2*PI for full circle)
    pub total_angle: f64,
}

#[derive(Debug)]
pub struct CircularPatternOp;

impl Operation for CircularPatternOp {
    type Params = CircularPatternParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "circular_pattern".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Circular Pattern"
    }
}
