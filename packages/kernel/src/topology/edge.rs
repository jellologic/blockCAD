use crate::id::EntityId;

use super::vertex::VertexId;

pub type EdgeId = EntityId<Edge>;

/// Orientation of a coedge relative to its edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Orientation {
    Forward,
    Reversed,
}

/// A topological edge connecting two vertices, with associated geometry (curve).
/// The curve is stored separately in the BRep and referenced by EdgeId.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Edge {
    pub start: VertexId,
    pub end: VertexId,
    /// Index into the BRep's curve storage
    pub curve_index: Option<usize>,
}

impl Edge {
    pub fn new(start: VertexId, end: VertexId) -> Self {
        Self {
            start,
            end,
            curve_index: None,
        }
    }

    pub fn with_curve(mut self, curve_index: usize) -> Self {
        self.curve_index = Some(curve_index);
        self
    }
}
