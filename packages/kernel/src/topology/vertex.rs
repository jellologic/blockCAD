use crate::geometry::Pt3;
use crate::id::EntityId;

pub type VertexId = EntityId<Vertex>;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Vertex {
    pub point: Pt3,
}

impl Vertex {
    pub fn new(point: Pt3) -> Self {
        Self { point }
    }
}
