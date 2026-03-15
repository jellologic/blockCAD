use super::{solid::SolidId, shell::ShellId, edge::EdgeId, vertex::VertexId};

/// The top-level topological entity representing a CAD body.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Body {
    Solid(SolidId),
    Sheet(ShellId),
    Wire(Vec<EdgeId>),
    Point(VertexId),
    Empty,
}
