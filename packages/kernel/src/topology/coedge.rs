use crate::id::EntityId;

use super::edge::{EdgeId, Orientation};

pub type CoEdgeId = EntityId<CoEdge>;

/// A co-edge (half-edge) used in loop traversal.
/// Each edge has up to two coedges (one per adjacent face).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CoEdge {
    pub edge: EdgeId,
    pub orientation: Orientation,
    /// Next coedge in the loop (forms a linked ring)
    pub next: Option<CoEdgeId>,
    /// Previous coedge in the loop
    pub prev: Option<CoEdgeId>,
    /// Twin coedge on the adjacent face (if any)
    pub twin: Option<CoEdgeId>,
}

impl CoEdge {
    pub fn new(edge: EdgeId, orientation: Orientation) -> Self {
        Self {
            edge,
            orientation,
            next: None,
            prev: None,
            twin: None,
        }
    }
}
