use crate::id::EntityId;

use super::coedge::CoEdgeId;

pub type LoopId = EntityId<Loop>;

/// A loop is an ordered ring of co-edges forming a closed boundary on a face.
/// A face has exactly one outer loop and zero or more inner loops (holes).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Loop {
    pub coedges: Vec<CoEdgeId>,
}

impl Loop {
    pub fn new(coedges: Vec<CoEdgeId>) -> Self {
        Self { coedges }
    }

    pub fn is_empty(&self) -> bool {
        self.coedges.is_empty()
    }

    pub fn len(&self) -> usize {
        self.coedges.len()
    }
}
