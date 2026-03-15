use crate::id::EntityId;

use super::shell::ShellId;

pub type SolidId = EntityId<Solid>;

/// A solid is a bounded volume defined by one or more shells.
/// The first shell is the outer boundary; additional shells are voids.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Solid {
    pub shells: Vec<ShellId>,
}

impl Solid {
    pub fn new(shells: Vec<ShellId>) -> Self {
        Self { shells }
    }
}
