use crate::error::{KernelError, KernelResult};
use crate::solver::graph::ConstraintGraph;

use super::sketch::Sketch;

/// Bridge between the Sketch data model and the constraint solver.
/// Converts sketch entities and constraints into solver variables and equations.
pub fn build_constraint_graph(_sketch: &Sketch) -> KernelResult<ConstraintGraph> {
    // TODO: Map sketch entities to solver variables:
    // - Each Point becomes 2 variables (x, y)
    // - Line endpoints are references to point variables
    // - Circle center + radius = 3 variables
    // - Arc center + start_angle + end_angle + radius = 4 variables
    //
    // Then map constraints to equations:
    // - Coincident: (x1-x2)²+(y1-y2)² = 0
    // - Distance: sqrt((x1-x2)²+(y1-y2)²) - d = 0
    // - Horizontal: y1-y2 = 0
    // - Vertical: x1-x2 = 0
    // - etc.
    Err(KernelError::Internal(
        "Solver bridge not yet implemented".into(),
    ))
}
