use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

/// A typed, generational index into an entity store.
/// The phantom type parameter prevents mixing IDs across entity types.
#[derive(Serialize, Deserialize)]
pub struct EntityId<T> {
    index: u32,
    generation: u32,
    #[serde(skip)]
    _phantom: PhantomData<fn() -> T>,
}

impl<T> EntityId<T> {
    pub fn new(index: u32, generation: u32) -> Self {
        Self {
            index,
            generation,
            _phantom: PhantomData,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }
}

impl<T> Clone for EntityId<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for EntityId<T> {}

impl<T> PartialEq for EntityId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

impl<T> Eq for EntityId<T> {}

impl<T> Hash for EntityId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
        self.generation.hash(state);
    }
}

impl<T> fmt::Debug for EntityId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "EntityId<{}>({}, gen={})",
            std::any::type_name::<T>(),
            self.index,
            self.generation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEntity;

    #[test]
    fn id_equality() {
        let a: EntityId<TestEntity> = EntityId::new(0, 0);
        let b: EntityId<TestEntity> = EntityId::new(0, 0);
        let c: EntityId<TestEntity> = EntityId::new(0, 1);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn id_is_copy() {
        let a: EntityId<TestEntity> = EntityId::new(1, 0);
        let b = a;
        assert_eq!(a, b); // a is still valid — Copy
    }

    #[test]
    fn id_debug_includes_type_name() {
        let id: EntityId<TestEntity> = EntityId::new(42, 7);
        let dbg = format!("{:?}", id);
        assert!(dbg.contains("42"));
        assert!(dbg.contains("gen=7"));
        assert!(dbg.contains("TestEntity"));
    }
}
