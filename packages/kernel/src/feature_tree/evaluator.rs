use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

use super::tree::FeatureTree;

/// Evaluate the feature tree, producing the final BRep.
///
/// Replays operations from the first dirty cache entry to the cursor.
/// Features marked as Suppressed are skipped.
pub fn evaluate(_tree: &mut FeatureTree) -> KernelResult<BRep> {
    // TODO: Implementation:
    // 1. Start from the last valid cached BRep (or empty BRep if none)
    // 2. For each active, non-suppressed feature from that point to cursor:
    //    a. Look up the Operation for the feature's FeatureKind
    //    b. Deserialize FeatureParams into the Operation's Params type
    //    c. Call operation.execute(params, &current_brep)
    //    d. Cache the result
    //    e. Mark feature as Evaluated (or Failed on error)
    // 3. Return the BRep at the cursor position
    Err(KernelError::Internal(
        "Feature tree evaluator not yet implemented".into(),
    ))
}
