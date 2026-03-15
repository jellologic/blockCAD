use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

use super::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RevolveParams {
    pub axis_origin: Pt3,
    pub axis_direction: Vec3,
    /// Angle of revolution in radians (2*PI for full revolution)
    pub angle: f64,
}

#[derive(Debug)]
pub struct RevolveOp;

impl Operation for RevolveOp {
    type Params = RevolveParams;

    fn execute(&self, _params: &Self::Params, _input: &BRep) -> KernelResult<BRep> {
        Err(KernelError::Operation {
            op: "revolve".into(),
            detail: "Not yet implemented".into(),
        })
    }

    fn name(&self) -> &'static str {
        "Revolve"
    }
}
