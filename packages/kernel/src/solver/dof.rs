use super::graph::ConstraintGraph;
use crate::assembly::Assembly;

/// Degree of freedom analysis result
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DofStatus {
    /// System is fully constrained (DOF = 0)
    FullyConstrained,
    /// System is under-constrained with remaining degrees of freedom
    UnderConstrained { dof: u32 },
    /// System is over-constrained (more equations than free variables)
    OverConstrained { redundant: u32 },
}

/// Per-component DOF analysis result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComponentDofInfo {
    pub component_id: String,
    pub component_name: String,
    pub status: DofStatus,
    /// Number of mates constraining this component.
    pub mate_count: u32,
    /// Is the component grounded (0 DOF by definition)?
    pub grounded: bool,
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

/// Analyze DOF per component in an assembly.
///
/// Each non-grounded component has 6 DOF (3 translation + 3 rotation).
/// Each mate removes some DOF depending on type.
/// Grounded components have 0 DOF.
pub fn analyze_assembly_dof(assembly: &Assembly) -> Vec<ComponentDofInfo> {
    let mut results = Vec::new();

    for comp in &assembly.components {
        if comp.suppressed {
            continue;
        }

        if comp.grounded {
            results.push(ComponentDofInfo {
                component_id: comp.id.clone(),
                component_name: comp.name.clone(),
                status: DofStatus::FullyConstrained,
                mate_count: 0,
                grounded: true,
            });
            continue;
        }

        // Count mates constraining this component
        let mates_for_comp: Vec<_> = assembly.mates.iter()
            .filter(|m| !m.suppressed && (m.component_a == comp.id || m.component_b == comp.id))
            .collect();

        let mate_count = mates_for_comp.len() as u32;

        // Estimate DOF removed per mate type
        let mut dof_removed: i64 = 0;
        for mate in &mates_for_comp {
            dof_removed += match &mate.kind {
                crate::assembly::MateKind::Coincident => 3, // removes 3 translational DOF
                crate::assembly::MateKind::Concentric => 4, // 2 translations + 2 rotations
                crate::assembly::MateKind::Distance { .. } => 1,
                crate::assembly::MateKind::Angle { .. } => 1,
                crate::assembly::MateKind::Parallel => 2,
                crate::assembly::MateKind::Perpendicular => 2,
                crate::assembly::MateKind::Tangent => 1,
                crate::assembly::MateKind::Lock => 6,
                crate::assembly::MateKind::Hinge => 5, // 1 rotational DOF remains
                crate::assembly::MateKind::Gear { .. } => 1,
                crate::assembly::MateKind::Screw { .. } => 5,
                crate::assembly::MateKind::Limit { .. } => 0, // limits don't remove DOF, just bound them
                crate::assembly::MateKind::Width => 1,
                crate::assembly::MateKind::Symmetric => 3,
            };
        }

        let remaining_dof = 6 - dof_removed;
        let status = if remaining_dof == 0 {
            DofStatus::FullyConstrained
        } else if remaining_dof > 0 {
            DofStatus::UnderConstrained { dof: remaining_dof as u32 }
        } else {
            DofStatus::OverConstrained { redundant: (-remaining_dof) as u32 }
        };

        results.push(ComponentDofInfo {
            component_id: comp.id.clone(),
            component_name: comp.name.clone(),
            status,
            mate_count,
            grounded: false,
        });
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Mate, MateKind, GeometryRef, Part};
    use crate::feature_tree::FeatureTree;

    fn make_assembly() -> Assembly {
        let mut asm = Assembly::new();
        asm.add_part(Part::new("p1", "Part", FeatureTree::new()));
        asm.add_component(
            Component::new("c1".into(), "p1".into(), "Ground".into())
                .with_grounded(true)
        );
        asm.add_component(Component::new("c2".into(), "p1".into(), "Free".into()));
        asm
    }

    #[test]
    fn empty_graph_is_fully_constrained() {
        let graph = ConstraintGraph::new();
        assert_eq!(analyze_dof(&graph), DofStatus::FullyConstrained);
    }

    #[test]
    fn grounded_component_has_zero_dof() {
        let asm = make_assembly();
        let analysis = analyze_assembly_dof(&asm);
        assert_eq!(analysis.len(), 2);
        assert!(analysis[0].grounded);
        assert_eq!(analysis[0].status, DofStatus::FullyConstrained);
    }

    #[test]
    fn free_component_has_six_dof() {
        let asm = make_assembly();
        let analysis = analyze_assembly_dof(&asm);
        assert_eq!(analysis[1].status, DofStatus::UnderConstrained { dof: 6 });
    }

    #[test]
    fn coincident_mate_removes_three_dof() {
        let mut asm = make_assembly();
        asm.mates.push(Mate {
            id: "m1".into(),
            kind: MateKind::Coincident,
            component_a: "c1".into(),
            component_b: "c2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });
        let analysis = analyze_assembly_dof(&asm);
        // c2 should have 6 - 3 = 3 DOF remaining
        assert_eq!(analysis[1].status, DofStatus::UnderConstrained { dof: 3 });
        assert_eq!(analysis[1].mate_count, 1);
    }
}
