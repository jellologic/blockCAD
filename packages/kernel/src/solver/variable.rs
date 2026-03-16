use crate::id::EntityId;

/// Marker type for variable IDs
#[derive(Debug)]
pub struct VariableMarker;
pub type VariableId = EntityId<VariableMarker>;

/// A solver variable representing a single degree of freedom.
#[derive(Debug, Clone)]
pub struct Variable {
    pub value: f64,
    pub fixed: bool,
}

impl Variable {
    pub fn new(value: f64) -> Self {
        Self {
            value,
            fixed: false,
        }
    }

    pub fn fixed(value: f64) -> Self {
        Self {
            value,
            fixed: true,
        }
    }
}

/// Storage for solver variables, accessible by VariableId.
#[derive(Debug, Default, Clone)]
pub struct VariableStore {
    variables: Vec<Variable>,
}

impl VariableStore {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
        }
    }

    pub fn add(&mut self, var: Variable) -> VariableId {
        let index = self.variables.len() as u32;
        self.variables.push(var);
        VariableId::new(index, 0)
    }

    pub fn get(&self, id: VariableId) -> Option<&Variable> {
        self.variables.get(id.index() as usize)
    }

    pub fn get_mut(&mut self, id: VariableId) -> Option<&mut Variable> {
        self.variables.get_mut(id.index() as usize)
    }

    pub fn value(&self, id: VariableId) -> f64 {
        self.variables[id.index() as usize].value
    }

    pub fn set_value(&mut self, id: VariableId, value: f64) {
        self.variables[id.index() as usize].value = value;
    }

    pub fn len(&self) -> usize {
        self.variables.len()
    }

    pub fn is_empty(&self) -> bool {
        self.variables.is_empty()
    }

    /// Count of free (non-fixed) variables
    pub fn free_count(&self) -> usize {
        self.variables.iter().filter(|v| !v.fixed).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variable_store_operations() {
        let mut store = VariableStore::new();
        let x = store.add(Variable::new(1.0));
        let y = store.add(Variable::new(2.0));
        let _z = store.add(Variable::fixed(3.0));

        assert_eq!(store.len(), 3);
        assert_eq!(store.free_count(), 2);
        assert_eq!(store.value(x), 1.0);

        store.set_value(y, 5.0);
        assert_eq!(store.value(y), 5.0);
    }
}
