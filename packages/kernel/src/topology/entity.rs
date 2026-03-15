use crate::error::{KernelError, KernelResult};

// Re-export the EntityId from id module
pub use crate::id::EntityId;

/// A generational arena for storing B-Rep entities.
/// Provides O(1) insert, lookup, and remove with generation tracking
/// to detect stale references.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")]
pub struct EntityStore<T> {
    entries: Vec<Entry<T>>,
    free_list: Vec<u32>,
    len: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "T: serde::Serialize + serde::de::DeserializeOwned")]
enum Entry<T> {
    Occupied { generation: u32, value: T },
    Vacant { generation: u32 },
}

impl<T> EntityStore<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free_list: Vec::new(),
            len: 0,
        }
    }

    pub fn insert(&mut self, value: T) -> EntityId<T> {
        self.len += 1;
        if let Some(index) = self.free_list.pop() {
            let generation = match &self.entries[index as usize] {
                Entry::Vacant { generation } => *generation,
                Entry::Occupied { .. } => unreachable!("Free list pointed to occupied entry"),
            };
            self.entries[index as usize] = Entry::Occupied { generation, value };
            EntityId::new(index, generation)
        } else {
            let index = self.entries.len() as u32;
            self.entries.push(Entry::Occupied {
                generation: 0,
                value,
            });
            EntityId::new(index, 0)
        }
    }

    pub fn get(&self, id: EntityId<T>) -> KernelResult<&T> {
        match self.entries.get(id.index() as usize) {
            Some(Entry::Occupied { generation, value }) if *generation == id.generation() => {
                Ok(value)
            }
            _ => Err(KernelError::NotFound(format!("{:?}", id))),
        }
    }

    pub fn get_mut(&mut self, id: EntityId<T>) -> KernelResult<&mut T> {
        match self.entries.get_mut(id.index() as usize) {
            Some(Entry::Occupied { generation, value }) if *generation == id.generation() => {
                Ok(value)
            }
            _ => Err(KernelError::NotFound(format!("{:?}", id))),
        }
    }

    pub fn remove(&mut self, id: EntityId<T>) -> KernelResult<T> {
        let index = id.index() as usize;
        match self.entries.get(index) {
            Some(Entry::Occupied { generation, .. }) if *generation == id.generation() => {}
            _ => return Err(KernelError::NotFound(format!("{:?}", id))),
        }
        let old = std::mem::replace(
            &mut self.entries[index],
            Entry::Vacant {
                generation: id.generation() + 1,
            },
        );
        self.free_list.push(id.index());
        self.len -= 1;
        match old {
            Entry::Occupied { value, .. } => Ok(value),
            Entry::Vacant { .. } => unreachable!(),
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterate over all occupied entries
    pub fn iter(&self) -> impl Iterator<Item = (EntityId<T>, &T)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(i, entry)| match entry {
                Entry::Occupied { generation, value } => {
                    Some((EntityId::new(i as u32, *generation), value))
                }
                Entry::Vacant { .. } => None,
            })
    }
}

impl<T> Default for EntityStore<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct TestItem(i32);

    #[test]
    fn insert_and_get() {
        let mut store = EntityStore::new();
        let id = store.insert(TestItem(42));
        assert_eq!(store.get(id).unwrap(), &TestItem(42));
    }

    #[test]
    fn remove_and_reuse_slot() {
        let mut store = EntityStore::new();
        let id1 = store.insert(TestItem(1));
        store.remove(id1).unwrap();
        let id2 = store.insert(TestItem(2));
        // Same index, different generation
        assert_eq!(id2.index(), id1.index());
        assert_ne!(id2.generation(), id1.generation());
    }

    #[test]
    fn stale_id_returns_not_found() {
        let mut store = EntityStore::new();
        let id = store.insert(TestItem(1));
        store.remove(id).unwrap();
        store.insert(TestItem(2)); // occupies same slot with new generation
        assert!(store.get(id).is_err()); // old ID is stale
    }

    #[test]
    fn len_tracking() {
        let mut store = EntityStore::new();
        assert_eq!(store.len(), 0);
        let id1 = store.insert(TestItem(1));
        let _id2 = store.insert(TestItem(2));
        assert_eq!(store.len(), 2);
        store.remove(id1).unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn iter_over_occupied() {
        let mut store = EntityStore::new();
        store.insert(TestItem(1));
        let id2 = store.insert(TestItem(2));
        store.insert(TestItem(3));
        store.remove(id2).unwrap();
        let items: Vec<i32> = store.iter().map(|(_, item)| item.0).collect();
        assert_eq!(items, vec![1, 3]);
    }
}
