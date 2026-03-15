use crate::error::KernelResult;
use crate::topology::BRep;

/// Every feature operation implements this trait.
///
/// Operations are pure functions: they consume parameters and
/// an existing BRep, returning a new BRep. No mutation.
/// This enables the feature tree to replay from any rollback point.
pub trait Operation: Send + Sync + std::fmt::Debug {
    type Params: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + std::fmt::Debug;

    /// Execute the operation, producing a new BRep from the input.
    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep>;

    /// Human-readable name for UI and serialization.
    fn name(&self) -> &'static str;
}
