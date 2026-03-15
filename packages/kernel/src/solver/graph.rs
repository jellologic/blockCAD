use super::equation::Equation;
use super::variable::VariableStore;

/// A constraint graph representing the system of equations to be solved.
/// Built from sketch entities and constraints by the solver bridge.
#[derive(Debug)]
pub struct ConstraintGraph {
    pub variables: VariableStore,
    pub equations: Vec<Box<dyn Equation>>,
}

impl ConstraintGraph {
    pub fn new() -> Self {
        Self {
            variables: VariableStore::new(),
            equations: Vec::new(),
        }
    }

    pub fn add_equation(&mut self, eq: Box<dyn Equation>) {
        self.equations.push(eq);
    }

    /// Number of equations in the system
    pub fn equation_count(&self) -> usize {
        self.equations.len()
    }

    /// Number of free variables
    pub fn free_variable_count(&self) -> usize {
        self.variables.free_count()
    }
}

impl Default for ConstraintGraph {
    fn default() -> Self {
        Self::new()
    }
}
