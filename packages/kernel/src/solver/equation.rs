use super::variable::{VariableId, VariableStore};

/// A scalar equation f(variables) = 0.
/// The solver builds a system of these from sketch constraints
/// and finds roots via Newton-Raphson iteration.
pub trait Equation: Send + Sync + std::fmt::Debug {
    /// Evaluate f at current variable values. Returns the residual.
    fn eval(&self, vars: &VariableStore) -> f64;

    /// Compute partial derivatives ∂f/∂xᵢ for all variables this equation references.
    /// Returns (variable_id, partial_derivative) pairs.
    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)>;

    /// The set of variable IDs this equation depends on.
    fn variable_ids(&self) -> &[VariableId];
}
