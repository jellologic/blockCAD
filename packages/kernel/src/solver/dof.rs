use super::graph::ConstraintGraph;

/// Degree of freedom analysis result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DofStatus {
    /// System is fully constrained (DOF = 0)
    FullyConstrained,
    /// System is under-constrained with remaining degrees of freedom
    UnderConstrained { dof: u32 },
    /// System is over-constrained (more equations than free variables)
    OverConstrained { redundant: u32 },
}

/// Analyze the degrees of freedom of a constraint graph.
///
/// Simple counting method: DOF = free_variables - equations
/// A more sophisticated analysis would check the Jacobian rank.
pub fn analyze_dof(graph: &ConstraintGraph) -> DofStatus {
    let free_vars = graph.free_variable_count() as i64;
    let equations = graph.equation_count() as i64;
    let dof = free_vars - equations;

    if dof == 0 {
        DofStatus::FullyConstrained
    } else if dof > 0 {
        DofStatus::UnderConstrained { dof: dof as u32 }
    } else {
        DofStatus::OverConstrained {
            redundant: (-dof) as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_graph_is_fully_constrained() {
        let graph = ConstraintGraph::new();
        assert_eq!(analyze_dof(&graph), DofStatus::FullyConstrained);
    }
}
