use crate::id::EntityId;

use super::face::FaceId;

pub type ShellId = EntityId<Shell>;

/// A shell is a connected set of faces. A closed shell bounds a volume.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Shell {
    pub faces: Vec<FaceId>,
    pub closed: bool,
}

impl Shell {
    pub fn new(faces: Vec<FaceId>, closed: bool) -> Self {
        Self { faces, closed }
    }
}
