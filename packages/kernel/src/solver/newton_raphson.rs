use crate::error::{KernelError, KernelResult};

use super::graph::ConstraintGraph;

/// Configuration for the Newton-Raphson solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub max_iterations: usize,
    pub tolerance: f64,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            tolerance: 1e-9,
        }
    }
}

/// Result of a solver run
#[derive(Debug)]
pub struct SolverResult {
    pub converged: bool,
    pub iterations: usize,
    pub residual: f64,
}

/// Solve the constraint system using Newton-Raphson iteration.
///
/// Algorithm:
/// 1. Evaluate residual vector f(x)
/// 2. Build Jacobian matrix J
/// 3. Solve J * Δx = -f for Δx (using faer LU decomposition)
/// 4. Update x += Δx (skip fixed variables)
/// 5. Repeat until ||f|| < tolerance or max iterations
pub fn solve(
    graph: &mut ConstraintGraph,
    config: &SolverConfig,
) -> KernelResult<SolverResult> {
    let n_eq = graph.equations.len();
    if n_eq == 0 {
        return Ok(SolverResult {
            converged: true,
            iterations: 0,
            residual: 0.0,
        });
    }

    // Build index map: variable index → column in Jacobian (skip fixed vars)
    let n_vars = graph.variables.len();
    let mut var_to_col: Vec<Option<usize>> = vec![None; n_vars];
    let mut col_to_var: Vec<usize> = Vec::new();
    for i in 0..n_vars {
        use super::variable::VariableId;
        let id = VariableId::new(i as u32, 0);
        if let Some(var) = graph.variables.get(id) {
            if !var.fixed {
                var_to_col[i] = Some(col_to_var.len());
                col_to_var.push(i);
            }
        }
    }
    let n_free = col_to_var.len();

    if n_free == 0 {
        // All variables fixed — just check if equations are satisfied
        let residual = eval_residual_norm(graph);
        return Ok(SolverResult {
            converged: residual < config.tolerance,
            iterations: 0,
            residual,
        });
    }

    for iteration in 0..config.max_iterations {
        // 1. Evaluate residual vector
        let mut residual_vec: Vec<f64> = Vec::with_capacity(n_eq);
        for eq in &graph.equations {
            residual_vec.push(eq.eval(&graph.variables));
        }

        // Check convergence
        let residual_norm: f64 = residual_vec.iter().map(|r| r * r).sum::<f64>().sqrt();
        if residual_norm < config.tolerance {
            return Ok(SolverResult {
                converged: true,
                iterations: iteration,
                residual: residual_norm,
            });
        }

        // 2. Build Jacobian matrix (dense, n_eq x n_free)
        let mut jacobian = vec![0.0f64; n_eq * n_free];
        for (row, eq) in graph.equations.iter().enumerate() {
            for (var_id, partial) in eq.jacobian_row(&graph.variables) {
                let var_idx = var_id.index() as usize;
                if let Some(col) = var_to_col.get(var_idx).copied().flatten() {
                    jacobian[row * n_free + col] = partial;
                }
            }
        }

        // 3. Solve J * Δx = -f using faer
        let delta = solve_linear_system(&jacobian, &residual_vec, n_eq, n_free)?;

        // 4. Update free variables: x += Δx
        for (col, &var_idx) in col_to_var.iter().enumerate() {
            use super::variable::VariableId;
            let id = VariableId::new(var_idx as u32, 0);
            let old = graph.variables.value(id);
            graph.variables.set_value(id, old - delta[col]);
        }
    }

    let final_residual = eval_residual_norm(graph);
    if final_residual < config.tolerance {
        Ok(SolverResult {
            converged: true,
            iterations: config.max_iterations,
            residual: final_residual,
        })
    } else {
        Err(KernelError::ConstraintSolver {
            reason: format!(
                "Did not converge after {} iterations (residual: {:.2e})",
                config.max_iterations, final_residual
            ),
            dof: None,
        })
    }
}

fn eval_residual_norm(graph: &ConstraintGraph) -> f64 {
    graph
        .equations
        .iter()
        .map(|eq| {
            let r = eq.eval(&graph.variables);
            r * r
        })
        .sum::<f64>()
        .sqrt()
}

/// Solve the linear system J * x = b using faer LU decomposition.
/// J is n_eq x n_free, b is n_eq.
///
/// For overdetermined systems (n_eq > n_free), solves the least-squares problem.
/// For square systems, direct LU.
fn solve_linear_system(
    j_flat: &[f64],
    b: &[f64],
    n_eq: usize,
    n_free: usize,
) -> KernelResult<Vec<f64>> {
    use faer::prelude::*;

    if n_eq == 0 || n_free == 0 {
        return Ok(vec![0.0; n_free]);
    }

    // Build faer matrix from flat row-major data
    let j = faer::Mat::from_fn(n_eq, n_free, |row, col| j_flat[row * n_free + col]);
    let b_vec = faer::Mat::from_fn(n_eq, 1, |row, _| b[row]);

    if n_eq == n_free {
        // Square system: use LU
        let lu = j.partial_piv_lu();
        let x = lu.solve(&b_vec);
        Ok((0..n_free).map(|i| x.read(i, 0)).collect())
    } else if n_eq > n_free {
        // Overdetermined: solve normal equations J^T * J * x = J^T * b
        let jt = j.transpose();
        let jtj = &jt * &j;
        let jtb = &jt * &b_vec;
        let lu = jtj.partial_piv_lu();
        let x = lu.solve(&jtb);
        Ok((0..n_free).map(|i| x.read(i, 0)).collect())
    } else {
        // Underdetermined — more variables than equations
        // For now, solve the minimum-norm solution via J^T * (J * J^T)^-1 * b
        let jt = j.transpose();
        let jjt = &j * &jt;
        let lu = jjt.partial_piv_lu();
        let y = lu.solve(&b_vec);
        let x = &jt * &y;
        Ok((0..n_free).map(|i| x.read(i, 0)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::equations::*;
    use crate::solver::graph::ConstraintGraph;
    use crate::solver::variable::Variable;

    #[test]
    fn default_config() {
        let config = SolverConfig::default();
        assert_eq!(config.max_iterations, 100);
        assert!((config.tolerance - 1e-9).abs() < 1e-15);
    }

    #[test]
    fn solve_fixed_point() {
        let mut graph = ConstraintGraph::new();
        let x = graph.variables.add(Variable::new(10.0)); // start at 10
        let y = graph.variables.add(Variable::new(-5.0)); // start at -5
        graph.add_equation(Box::new(FixedEquation::new(x, 3.0)));
        graph.add_equation(Box::new(FixedEquation::new(y, 7.0)));

        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged);
        assert!((graph.variables.value(x) - 3.0).abs() < 1e-9);
        assert!((graph.variables.value(y) - 7.0).abs() < 1e-9);
    }

    #[test]
    fn solve_distance_constraint() {
        let mut graph = ConstraintGraph::new();
        // Point A fixed at origin
        let x1 = graph.variables.add(Variable::fixed(0.0));
        let y1 = graph.variables.add(Variable::fixed(0.0));
        // Point B starts at (1, 0), should move to distance 5
        let x2 = graph.variables.add(Variable::new(1.0));
        let y2 = graph.variables.add(Variable::new(0.0));

        graph.add_equation(Box::new(DistanceEquation::new(x1, y1, x2, y2, 5.0)));

        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged);

        let dx = graph.variables.value(x2);
        let dy = graph.variables.value(y2);
        let dist = (dx * dx + dy * dy).sqrt();
        assert!(
            (dist - 5.0).abs() < 1e-6,
            "Expected distance 5.0, got {}",
            dist
        );
    }

    #[test]
    fn solve_rectangle() {
        // 4 points forming a rectangle:
        // p0=(0,0) fixed, p1=(w,0), p2=(w,h), p3=(0,h)
        // Constraints: p0 fixed, horizontal(p0-p1), vertical(p1-p2),
        //              horizontal(p2-p3), vertical(p3-p0),
        //              distance(p0-p1)=10, distance(p1-p2)=5
        let mut graph = ConstraintGraph::new();

        // Variables: x0,y0, x1,y1, x2,y2, x3,y3
        let x0 = graph.variables.add(Variable::fixed(0.0));
        let y0 = graph.variables.add(Variable::fixed(0.0));
        let x1 = graph.variables.add(Variable::new(8.0)); // initial guess
        let y1 = graph.variables.add(Variable::new(0.5));
        let x2 = graph.variables.add(Variable::new(8.0));
        let y2 = graph.variables.add(Variable::new(4.0));
        let x3 = graph.variables.add(Variable::new(0.5));
        let y3 = graph.variables.add(Variable::new(4.0));

        // Horizontal: p0-p1 (y0 == y1)
        graph.add_equation(Box::new(CoincidentEquation::new(y0, y1)));
        // Vertical: p1-p2 (x1 == x2)
        graph.add_equation(Box::new(CoincidentEquation::new(x1, x2)));
        // Horizontal: p2-p3 (y2 == y3)
        graph.add_equation(Box::new(CoincidentEquation::new(y2, y3)));
        // Vertical: p3-p0 (x3 == x0)
        graph.add_equation(Box::new(CoincidentEquation::new(x3, x0)));
        // Width = 10
        graph.add_equation(Box::new(DistanceEquation::new(x0, y0, x1, y1, 10.0)));
        // Height = 5
        graph.add_equation(Box::new(DistanceEquation::new(x1, y1, x2, y2, 5.0)));

        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver did not converge");

        // Verify rectangle dimensions
        let w = graph.variables.value(x1);
        let h = graph.variables.value(y2);
        assert!((w - 10.0).abs() < 1e-6, "Width: expected 10, got {}", w);
        assert!((h - 5.0).abs() < 1e-6, "Height: expected 5, got {}", h);

        // Verify corners
        assert!((graph.variables.value(x0) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(y0) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(x2) - 10.0).abs() < 1e-6);
        assert!((graph.variables.value(y2) - 5.0).abs() < 1e-6);
        assert!((graph.variables.value(x3) - 0.0).abs() < 1e-6);
        assert!((graph.variables.value(y3) - 5.0).abs() < 1e-6);
    }

    #[test]
    fn empty_system_converges() {
        let mut graph = ConstraintGraph::new();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged);
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn solver_converges_for_large_system() {
        // Stress test: a chain of 10 points along a line, each pair connected
        // by a horizontal constraint and a distance constraint. This creates
        // a large system (~20 equations) that exercises the solver at scale.
        let mut graph = ConstraintGraph::new();

        // 10 points: first fixed at origin, each subsequent ~10 units right
        let mut xs = Vec::new();
        let mut ys = Vec::new();
        for i in 0..10 {
            let x = graph.variables.add(Variable::new(i as f64 * 9.5 + 0.5)); // slightly off
            let y = graph.variables.add(Variable::new(0.3 * (i as f64))); // slightly off horizontal
            if i == 0 {
                // Fix origin
                graph.add_equation(Box::new(FixedEquation::new(x, 0.0)));
                graph.add_equation(Box::new(FixedEquation::new(y, 0.0)));
            }
            xs.push(x);
            ys.push(y);
        }

        // Chain constraints: horizontal (y_i == y_{i+1}) and distance = 10
        for i in 0..9 {
            graph.add_equation(Box::new(CoincidentEquation::new(ys[i], ys[i + 1])));
            graph.add_equation(Box::new(DistanceEquation::new(xs[i], ys[i], xs[i + 1], ys[i + 1], 10.0)));
        }

        // 2 (fixed) + 9 (horiz) + 9 (dist) = 20 equations
        assert_eq!(graph.equation_count(), 20, "Should have 20 equations");

        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge for chain of 10 points (iterations={})", result.iterations);

        // Verify: all y should be 0, x[i] should be i*10
        for i in 0..10 {
            let xi = graph.variables.value(xs[i]);
            let yi = graph.variables.value(ys[i]);
            assert!((xi - i as f64 * 10.0).abs() < 1e-6, "x[{}] = {}, expected {}", i, xi, i as f64 * 10.0);
            assert!(yi.abs() < 1e-6, "y[{}] = {}, expected 0", i, yi);
        }
    }

    #[test]
    fn solver_handles_large_coordinates() {
        // Rectangle at (1e5, 1e5) scale
        use crate::sketch::constraint::{Constraint, ConstraintKind};
        use crate::sketch::entity::SketchEntity;
        use crate::sketch::sketch::Sketch;
        use crate::sketch::solver_bridge::build_constraint_graph;
        use crate::geometry::surface::plane::Plane;
        use crate::geometry::Pt2;

        let offset = 1e5;
        let mut sketch = Sketch::new(Plane::xy(0.0));

        let p0 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(offset, offset) });
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(offset + 8.0, offset + 0.5) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(offset + 8.0, offset + 4.0) });
        let p3 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(offset + 0.5, offset + 4.0) });

        let bottom = sketch.add_entity(SketchEntity::Line { start: p0, end: p1 });
        let right = sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        let top = sketch.add_entity(SketchEntity::Line { start: p2, end: p3 });
        let left = sketch.add_entity(SketchEntity::Line { start: p3, end: p0 });

        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p0]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![bottom]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Horizontal, vec![top]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![right]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Vertical, vec![left]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 10.0 }, vec![p0, p1]));
        sketch.add_constraint(Constraint::new(ConstraintKind::Distance { value: 5.0 }, vec![p1, p2]));

        let (mut graph, var_map) = build_constraint_graph(&sketch).unwrap();
        let result = solve(&mut graph, &SolverConfig::default()).unwrap();
        assert!(result.converged, "Solver should converge at large coords");

        // Verify positions
        let (x0, y0) = var_map.point_vars(p0).unwrap();
        assert!((graph.variables.value(x0) - offset).abs() < 1e-4);
        assert!((graph.variables.value(y0) - offset).abs() < 1e-4);
        let (x1, _) = var_map.point_vars(p1).unwrap();
        assert!((graph.variables.value(x1) - (offset + 10.0)).abs() < 1e-4);
    }
}
