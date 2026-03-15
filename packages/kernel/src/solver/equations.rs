//! Concrete equation implementations for sketch constraint solving.
//!
//! Each equation represents a scalar function f(x) = 0.
//! The solver finds variable values where all equations are simultaneously zero.

use super::equation::Equation;
use super::variable::{VariableId, VariableStore};

/// Fixed point: x - target = 0
#[derive(Debug)]
pub struct FixedEquation {
    pub var: VariableId,
    pub target: f64,
    ids: Vec<VariableId>,
}

impl FixedEquation {
    pub fn new(var: VariableId, target: f64) -> Self {
        Self {
            var,
            target,
            ids: vec![var],
        }
    }
}

impl Equation for FixedEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        vars.value(self.var) - self.target
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![(self.var, 1.0)]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Coincident (same coordinate): x1 - x2 = 0
#[derive(Debug)]
pub struct CoincidentEquation {
    pub var_a: VariableId,
    pub var_b: VariableId,
    ids: Vec<VariableId>,
}

impl CoincidentEquation {
    pub fn new(var_a: VariableId, var_b: VariableId) -> Self {
        Self {
            var_a,
            var_b,
            ids: vec![var_a, var_b],
        }
    }
}

impl Equation for CoincidentEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        vars.value(self.var_a) - vars.value(self.var_b)
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![(self.var_a, 1.0), (self.var_b, -1.0)]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Horizontal constraint: y1 - y2 = 0
/// (Same as CoincidentEquation but semantically different — applied to y-coordinates)
pub type HorizontalEquation = CoincidentEquation;

/// Vertical constraint: x1 - x2 = 0
/// (Same as CoincidentEquation but applied to x-coordinates)
pub type VerticalEquation = CoincidentEquation;

/// Distance constraint: (x1-x2)² + (y1-y2)² - d² = 0
///
/// Uses the squared form to avoid sqrt singularity at zero distance.
#[derive(Debug)]
pub struct DistanceEquation {
    pub x1: VariableId,
    pub y1: VariableId,
    pub x2: VariableId,
    pub y2: VariableId,
    pub distance_sq: f64,
    ids: Vec<VariableId>,
}

impl DistanceEquation {
    pub fn new(
        x1: VariableId,
        y1: VariableId,
        x2: VariableId,
        y2: VariableId,
        distance: f64,
    ) -> Self {
        Self {
            x1,
            y1,
            x2,
            y2,
            distance_sq: distance * distance,
            ids: vec![x1, y1, x2, y2],
        }
    }
}

impl Equation for DistanceEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx = vars.value(self.x1) - vars.value(self.x2);
        let dy = vars.value(self.y1) - vars.value(self.y2);
        dx * dx + dy * dy - self.distance_sq
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx = vars.value(self.x1) - vars.value(self.x2);
        let dy = vars.value(self.y1) - vars.value(self.y2);
        vec![
            (self.x1, 2.0 * dx),
            (self.y1, 2.0 * dy),
            (self.x2, -2.0 * dx),
            (self.y2, -2.0 * dy),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Perpendicular constraint between two line segments:
/// (x2-x1)*(x4-x3) + (y2-y1)*(y4-y3) = 0 (dot product = 0)
#[derive(Debug)]
pub struct PerpendicularEquation {
    pub x1: VariableId,
    pub y1: VariableId,
    pub x2: VariableId,
    pub y2: VariableId,
    pub x3: VariableId,
    pub y3: VariableId,
    pub x4: VariableId,
    pub y4: VariableId,
    ids: Vec<VariableId>,
}

impl PerpendicularEquation {
    pub fn new(
        x1: VariableId, y1: VariableId,
        x2: VariableId, y2: VariableId,
        x3: VariableId, y3: VariableId,
        x4: VariableId, y4: VariableId,
    ) -> Self {
        Self {
            x1, y1, x2, y2, x3, y3, x4, y4,
            ids: vec![x1, y1, x2, y2, x3, y3, x4, y4],
        }
    }
}

impl Equation for PerpendicularEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        dx1 * dx2 + dy1 * dy2
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        vec![
            (self.x1, -dx2),
            (self.y1, -dy2),
            (self.x2, dx2),
            (self.y2, dy2),
            (self.x3, -dx1),
            (self.y3, -dy1),
            (self.x4, dx1),
            (self.y4, dy1),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_store(values: &[f64]) -> (VariableStore, Vec<VariableId>) {
        let mut store = VariableStore::new();
        let ids: Vec<_> = values
            .iter()
            .map(|&v| store.add(super::super::variable::Variable::new(v)))
            .collect();
        (store, ids)
    }

    #[test]
    fn fixed_equation() {
        let (store, ids) = make_store(&[3.0]);
        let eq = FixedEquation::new(ids[0], 5.0);
        assert_eq!(eq.eval(&store), -2.0); // 3 - 5 = -2
        let jac = eq.jacobian_row(&store);
        assert_eq!(jac.len(), 1);
        assert_eq!(jac[0].1, 1.0);
    }

    #[test]
    fn coincident_equation() {
        let (store, ids) = make_store(&[3.0, 7.0]);
        let eq = CoincidentEquation::new(ids[0], ids[1]);
        assert_eq!(eq.eval(&store), -4.0); // 3 - 7 = -4
    }

    #[test]
    fn distance_equation_satisfied() {
        // Points at (0,0) and (3,4) → distance should be 5
        let (store, ids) = make_store(&[0.0, 0.0, 3.0, 4.0]);
        let eq = DistanceEquation::new(ids[0], ids[1], ids[2], ids[3], 5.0);
        let residual = eq.eval(&store);
        assert!(residual.abs() < 1e-9, "Expected 0, got {}", residual);
    }

    #[test]
    fn distance_equation_not_satisfied() {
        // Points at (0,0) and (1,0) → distance is 1, constraint says 5
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0]);
        let eq = DistanceEquation::new(ids[0], ids[1], ids[2], ids[3], 5.0);
        let residual = eq.eval(&store);
        assert_eq!(residual, 1.0 - 25.0); // 1² - 5² = -24
    }

    #[test]
    fn distance_jacobian_finite_difference() {
        let (store, ids) = make_store(&[1.0, 2.0, 4.0, 6.0]);
        let eq = DistanceEquation::new(ids[0], ids[1], ids[2], ids[3], 5.0);
        let jac = eq.jacobian_row(&store);

        // Verify via finite differences
        let eps = 1e-7;
        for &(var_id, analytic) in &jac {
            let mut store_plus = VariableStore::new();
            for (i, &v) in [1.0, 2.0, 4.0, 6.0].iter().enumerate() {
                let val = if i == var_id.index() as usize { v + eps } else { v };
                store_plus.add(super::super::variable::Variable::new(val));
            }
            let f_plus = eq.eval(&store_plus);
            let f_base = eq.eval(&store);
            let numerical = (f_plus - f_base) / eps;
            assert!(
                (analytic - numerical).abs() < 1e-5,
                "Jacobian mismatch for var {:?}: analytic={}, numerical={}",
                var_id,
                analytic,
                numerical
            );
        }
    }

    #[test]
    fn perpendicular_equation_satisfied() {
        // Line1: (0,0)→(1,0) (horizontal), Line2: (0,0)→(0,1) (vertical)
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let eq = PerpendicularEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn perpendicular_equation_not_satisfied() {
        // Two parallel horizontal lines
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
        let eq = PerpendicularEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!((eq.eval(&store) - 1.0).abs() < 1e-9); // dot product = 1
    }
}
