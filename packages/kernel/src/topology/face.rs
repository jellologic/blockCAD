use crate::id::EntityId;

use super::loop_::LoopId;

pub type FaceId = EntityId<Face>;

/// A topological face bounded by loops, with associated surface geometry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Face {
    /// Index into the BRep's surface storage
    pub surface_index: Option<usize>,
    /// The outer boundary loop
    pub outer_loop: Option<LoopId>,
    /// Inner loops (holes, cutouts)
    pub inner_loops: Vec<LoopId>,
    /// Whether the surface normal agrees with the face normal
    pub same_sense: bool,
}

impl Face {
    pub fn new() -> Self {
        Self {
            surface_index: None,
            outer_loop: None,
            inner_loops: Vec::new(),
            same_sense: true,
        }
    }

    pub fn with_surface(mut self, surface_index: usize) -> Self {
        self.surface_index = Some(surface_index);
        self
    }

    pub fn with_outer_loop(mut self, loop_id: LoopId) -> Self {
        self.outer_loop = Some(loop_id);
        self
    }

    pub fn with_inner_loop(mut self, loop_id: LoopId) -> Self {
        self.inner_loops.push(loop_id);
        self
    }
}

impl Default for Face {
    fn default() -> Self {
        Self::new()
    }
}
