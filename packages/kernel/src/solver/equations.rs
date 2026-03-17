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

/// Parallel constraint between two line segments:
/// (x2-x1)*(y4-y3) - (y2-y1)*(x4-x3) = 0 (cross product = 0)
#[derive(Debug)]
pub struct ParallelEquation {
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

impl ParallelEquation {
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

impl Equation for ParallelEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        dx1 * dy2 - dy1 * dx2
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        vec![
            (self.x1, -dy2),
            (self.y1, dx2),
            (self.x2, dy2),
            (self.y2, -dx2),
            (self.x3, dy1),
            (self.y3, -dx1),
            (self.x4, -dy1),
            (self.y4, dx1),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Collinear constraint: point C lies on line AB.
/// (Bx-Ax)*(Cy-Ay) - (By-Ay)*(Cx-Ax) = 0
#[derive(Debug)]
pub struct CollinearEquation {
    pub ax: VariableId,
    pub ay: VariableId,
    pub bx: VariableId,
    pub by: VariableId,
    pub cx: VariableId,
    pub cy: VariableId,
    ids: Vec<VariableId>,
}

impl CollinearEquation {
    pub fn new(
        ax: VariableId, ay: VariableId,
        bx: VariableId, by: VariableId,
        cx: VariableId, cy: VariableId,
    ) -> Self {
        Self {
            ax, ay, bx, by, cx, cy,
            ids: vec![ax, ay, bx, by, cx, cy],
        }
    }
}

impl Equation for CollinearEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let abx = vars.value(self.bx) - vars.value(self.ax);
        let aby = vars.value(self.by) - vars.value(self.ay);
        let acx = vars.value(self.cx) - vars.value(self.ax);
        let acy = vars.value(self.cy) - vars.value(self.ay);
        abx * acy - aby * acx
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        // f = (bx-ax)*(cy-ay) - (by-ay)*(cx-ax)
        let abx = vars.value(self.bx) - vars.value(self.ax);
        let aby = vars.value(self.by) - vars.value(self.ay);
        let acy = vars.value(self.cy) - vars.value(self.ay);
        let acx = vars.value(self.cx) - vars.value(self.ax);
        // df/dax = -(cy-ay) + (by-ay) = aby - acy
        // df/day = (cx-ax) - (bx-ax) = acx - abx
        // df/dbx = (cy-ay) = acy
        // df/dby = -(cx-ax) = -acx
        // df/dcx = -(by-ay) = -aby
        // df/dcy = (bx-ax) = abx
        vec![
            (self.ax, aby - acy),
            (self.ay, acx - abx),
            (self.bx, acy),
            (self.by, -acx),
            (self.cx, -aby),
            (self.cy, abx),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Angle constraint between two line segments:
/// sin(θ)*(dx1*dx2 + dy1*dy2) - cos(θ)*(dx1*dy2 - dy1*dx2) = 0
///
/// This avoids atan2 in the solver by using the identity:
/// sin(target)*dot - cos(target)*cross = 0 ↔ angle between lines = target
#[derive(Debug)]
pub struct AngleEquation {
    pub x1: VariableId,
    pub y1: VariableId,
    pub x2: VariableId,
    pub y2: VariableId,
    pub x3: VariableId,
    pub y3: VariableId,
    pub x4: VariableId,
    pub y4: VariableId,
    pub sin_target: f64,
    pub cos_target: f64,
    ids: Vec<VariableId>,
}

impl AngleEquation {
    pub fn new(
        x1: VariableId, y1: VariableId,
        x2: VariableId, y2: VariableId,
        x3: VariableId, y3: VariableId,
        x4: VariableId, y4: VariableId,
        angle: f64,
    ) -> Self {
        Self {
            x1, y1, x2, y2, x3, y3, x4, y4,
            sin_target: angle.sin(),
            cos_target: angle.cos(),
            ids: vec![x1, y1, x2, y2, x3, y3, x4, y4],
        }
    }
}

impl Equation for AngleEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        let dot = dx1 * dx2 + dy1 * dy2;
        let cross = dx1 * dy2 - dy1 * dx2;
        self.sin_target * dot - self.cos_target * cross
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        let s = self.sin_target;
        let c = self.cos_target;
        // d/d(x1) = s*(-dx2) - c*(-dy2) = -s*dx2 + c*dy2
        // d/d(y1) = s*(-dy2) - c*(dx2) = -s*dy2 - c*dx2
        // d/d(x2) = s*dx2 - c*dy2
        // d/d(y2) = s*dy2 + c*dx2
        // d/d(x3) = s*(-dx1) - c*(dy1) = -s*dx1 - c*dy1
        // d/d(y3) = s*(-dy1) - c*(-dx1) = -s*dy1 + c*dx1
        // d/d(x4) = s*dx1 + c*dy1
        // d/d(y4) = s*dy1 - c*dx1
        vec![
            (self.x1, -s * dx2 + c * dy2),
            (self.y1, -s * dy2 - c * dx2),
            (self.x2, s * dx2 - c * dy2),
            (self.y2, s * dy2 + c * dx2),
            (self.x3, -s * dx1 - c * dy1),
            (self.y3, -s * dy1 + c * dx1),
            (self.x4, s * dx1 + c * dy1),
            (self.y4, s * dy1 - c * dx1),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Midpoint constraint: point C is at the midpoint of A and B.
/// Produces TWO equations: Cx - (Ax+Bx)/2 = 0 and Cy - (Ay+By)/2 = 0
/// Use `MidpointEquationX` and `MidpointEquationY` separately.
#[derive(Debug)]
pub struct MidpointEquation {
    pub a: VariableId,
    pub b: VariableId,
    pub mid: VariableId,
    ids: Vec<VariableId>,
}

impl MidpointEquation {
    pub fn new(a: VariableId, b: VariableId, mid: VariableId) -> Self {
        Self {
            a,
            b,
            mid,
            ids: vec![a, b, mid],
        }
    }
}

impl Equation for MidpointEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        vars.value(self.mid) - (vars.value(self.a) + vars.value(self.b)) / 2.0
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![
            (self.a, -0.5),
            (self.b, -0.5),
            (self.mid, 1.0),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Radius constraint: r - target = 0
#[derive(Debug)]
pub struct RadiusEquation {
    pub r: VariableId,
    pub target: f64,
    ids: Vec<VariableId>,
}

impl RadiusEquation {
    pub fn new(r: VariableId, target: f64) -> Self {
        Self {
            r,
            target,
            ids: vec![r],
        }
    }
}

impl Equation for RadiusEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        vars.value(self.r) - self.target
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![(self.r, 1.0)]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Equal length constraint between two line segments:
/// (x2-x1)² + (y2-y1)² - (x4-x3)² - (y4-y3)² = 0
#[derive(Debug)]
pub struct EqualLengthEquation {
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

impl EqualLengthEquation {
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

impl Equation for EqualLengthEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        (dx1 * dx1 + dy1 * dy1) - (dx2 * dx2 + dy2 * dy2)
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx1 = vars.value(self.x2) - vars.value(self.x1);
        let dy1 = vars.value(self.y2) - vars.value(self.y1);
        let dx2 = vars.value(self.x4) - vars.value(self.x3);
        let dy2 = vars.value(self.y4) - vars.value(self.y3);
        vec![
            (self.x1, -2.0 * dx1),
            (self.y1, -2.0 * dy1),
            (self.x2, 2.0 * dx1),
            (self.y2, 2.0 * dy1),
            (self.x3, 2.0 * dx2),
            (self.y3, 2.0 * dy2),
            (self.x4, -2.0 * dx2),
            (self.y4, -2.0 * dy2),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Symmetric constraint: two points P1 and P2 are symmetric about an axis line.
/// Two conditions:
/// 1. Midpoint of P1,P2 lies on the axis: (mid - A) × axis_dir = 0
/// 2. P1→P2 is perpendicular to axis: (P2 - P1) · axis_dir = 0
///
/// Use `SymmetricMidpointEquation` and `SymmetricPerpendicularEquation` together.
///
/// Midpoint on axis: ((p1x+p2x)/2 - ax)*(by-ay) - ((p1y+p2y)/2 - ay)*(bx-ax) = 0
#[derive(Debug)]
pub struct SymmetricMidpointEquation {
    pub p1x: VariableId,
    pub p1y: VariableId,
    pub p2x: VariableId,
    pub p2y: VariableId,
    pub ax: VariableId,
    pub ay: VariableId,
    pub bx: VariableId,
    pub by: VariableId,
    ids: Vec<VariableId>,
}

impl SymmetricMidpointEquation {
    pub fn new(
        p1x: VariableId, p1y: VariableId,
        p2x: VariableId, p2y: VariableId,
        ax: VariableId, ay: VariableId,
        bx: VariableId, by: VariableId,
    ) -> Self {
        Self {
            p1x, p1y, p2x, p2y, ax, ay, bx, by,
            ids: vec![p1x, p1y, p2x, p2y, ax, ay, bx, by],
        }
    }
}

impl Equation for SymmetricMidpointEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let mx = (vars.value(self.p1x) + vars.value(self.p2x)) / 2.0;
        let my = (vars.value(self.p1y) + vars.value(self.p2y)) / 2.0;
        let ax = vars.value(self.ax);
        let ay = vars.value(self.ay);
        let dx = vars.value(self.bx) - ax;
        let dy = vars.value(self.by) - ay;
        (mx - ax) * dy - (my - ay) * dx
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx = vars.value(self.bx) - vars.value(self.ax);
        let dy = vars.value(self.by) - vars.value(self.ay);
        let mx = (vars.value(self.p1x) + vars.value(self.p2x)) / 2.0;
        let my = (vars.value(self.p1y) + vars.value(self.p2y)) / 2.0;
        let ax = vars.value(self.ax);
        let ay = vars.value(self.ay);
        vec![
            (self.p1x, dy / 2.0),
            (self.p1y, -dx / 2.0),
            (self.p2x, dy / 2.0),
            (self.p2y, -dx / 2.0),
            (self.ax, -dy + (my - ay)),
            (self.ay, dx - (mx - ax)),
            (self.bx, -(my - ay)),
            (self.by, mx - ax),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// P1→P2 perpendicular to axis direction:
/// (p2x-p1x)*(bx-ax) + (p2y-p1y)*(by-ay) = 0
#[derive(Debug)]
pub struct SymmetricPerpendicularEquation {
    pub p1x: VariableId,
    pub p1y: VariableId,
    pub p2x: VariableId,
    pub p2y: VariableId,
    pub ax: VariableId,
    pub ay: VariableId,
    pub bx: VariableId,
    pub by: VariableId,
    ids: Vec<VariableId>,
}

impl SymmetricPerpendicularEquation {
    pub fn new(
        p1x: VariableId, p1y: VariableId,
        p2x: VariableId, p2y: VariableId,
        ax: VariableId, ay: VariableId,
        bx: VariableId, by: VariableId,
    ) -> Self {
        Self {
            p1x, p1y, p2x, p2y, ax, ay, bx, by,
            ids: vec![p1x, p1y, p2x, p2y, ax, ay, bx, by],
        }
    }
}

impl Equation for SymmetricPerpendicularEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dpx = vars.value(self.p2x) - vars.value(self.p1x);
        let dpy = vars.value(self.p2y) - vars.value(self.p1y);
        let dx = vars.value(self.bx) - vars.value(self.ax);
        let dy = vars.value(self.by) - vars.value(self.ay);
        dpx * dx + dpy * dy
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dpx = vars.value(self.p2x) - vars.value(self.p1x);
        let dpy = vars.value(self.p2y) - vars.value(self.p1y);
        let dx = vars.value(self.bx) - vars.value(self.ax);
        let dy = vars.value(self.by) - vars.value(self.ay);
        vec![
            (self.p1x, -dx),
            (self.p1y, -dy),
            (self.p2x, dx),
            (self.p2y, dy),
            (self.ax, -dpx),
            (self.ay, -dpy),
            (self.bx, dpx),
            (self.by, dpy),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Concentric constraint (single axis): cx1 - cx2 = 0
/// Used in pairs (one for x, one for y) to force two circles/arcs to share the same center.
pub type ConcentricEquation = CoincidentEquation;

/// Point-on-line constraint: point (px, py) lies on line from (ax, ay) to (bx, by).
/// Uses the cross product form: (bx-ax)*(py-ay) - (by-ay)*(px-ax) = 0
/// This is equivalent to CollinearEquation but named for clarity.
pub type PointOnLineEquation = CollinearEquation;

/// Point-on-circle constraint: distance from point to center equals radius.
/// (px - cx)² + (py - cy)² - r² = 0
#[derive(Debug)]
pub struct PointOnCircleEquation {
    pub px: VariableId,
    pub py: VariableId,
    pub cx: VariableId,
    pub cy: VariableId,
    pub r: VariableId,
    ids: Vec<VariableId>,
}

impl PointOnCircleEquation {
    pub fn new(
        px: VariableId, py: VariableId,
        cx: VariableId, cy: VariableId,
        r: VariableId,
    ) -> Self {
        Self {
            px, py, cx, cy, r,
            ids: vec![px, py, cx, cy, r],
        }
    }
}

impl Equation for PointOnCircleEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx = vars.value(self.px) - vars.value(self.cx);
        let dy = vars.value(self.py) - vars.value(self.cy);
        let r = vars.value(self.r);
        dx * dx + dy * dy - r * r
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx = vars.value(self.px) - vars.value(self.cx);
        let dy = vars.value(self.py) - vars.value(self.cy);
        let r = vars.value(self.r);
        vec![
            (self.px, 2.0 * dx),
            (self.py, 2.0 * dy),
            (self.cx, -2.0 * dx),
            (self.cy, -2.0 * dy),
            (self.r, -2.0 * r),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Tangent constraint between a line and a circle:
/// The perpendicular distance from the circle center to the line equals the radius.
///
/// Given line (ax,ay)→(bx,by) and circle center (cx,cy) radius r:
/// cross = (bx-ax)*(cy-ay) - (by-ay)*(cx-ax)
/// len2 = (bx-ax)² + (by-ay)²
/// f = cross² - r² * len2 = 0
///
/// This avoids sqrt while enforcing |cross|/sqrt(len2) = r.
#[derive(Debug)]
pub struct TangentLineCircleEquation {
    pub ax: VariableId,
    pub ay: VariableId,
    pub bx: VariableId,
    pub by: VariableId,
    pub cx: VariableId,
    pub cy: VariableId,
    pub r: VariableId,
    ids: Vec<VariableId>,
}

impl TangentLineCircleEquation {
    pub fn new(
        ax: VariableId, ay: VariableId,
        bx: VariableId, by: VariableId,
        cx: VariableId, cy: VariableId,
        r: VariableId,
    ) -> Self {
        Self {
            ax, ay, bx, by, cx, cy, r,
            ids: vec![ax, ay, bx, by, cx, cy, r],
        }
    }
}

impl Equation for TangentLineCircleEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let dx = vars.value(self.bx) - vars.value(self.ax);
        let dy = vars.value(self.by) - vars.value(self.ay);
        let acx = vars.value(self.cx) - vars.value(self.ax);
        let acy = vars.value(self.cy) - vars.value(self.ay);
        let cross = dx * acy - dy * acx;
        let len2 = dx * dx + dy * dy;
        let r = vars.value(self.r);
        cross * cross - r * r * len2
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        let dx = vars.value(self.bx) - vars.value(self.ax);
        let dy = vars.value(self.by) - vars.value(self.ay);
        let acx = vars.value(self.cx) - vars.value(self.ax);
        let acy = vars.value(self.cy) - vars.value(self.ay);
        let cross = dx * acy - dy * acx;
        let len2 = dx * dx + dy * dy;
        let r = vars.value(self.r);
        let r2 = r * r;

        // f = cross² - r²·len2
        // cross = dx·acy - dy·acx
        // len2 = dx² + dy²
        //
        // dcross/dax = -acy + 0·acx ... let's do it carefully:
        // dx = bx-ax, dy = by-ay, acx = cx-ax, acy = cy-ay
        //
        // dcross/dax = d/dax[ (bx-ax)(cy-ay) - (by-ay)(cx-ax) ]
        //            = -(cy-ay) - (-(by-ay)) = -acy + dy
        // dcross/day = (bx-ax)·(-1) - (-1)·(cx-ax) = -dx + acx
        // dcross/dbx = acy
        // dcross/dby = -acx
        // dcross/dcx = -dy   [d/dcx of -(by-ay)(cx-ax) = -dy]
        // dcross/dcy = dx    [d/dcy of (bx-ax)(cy-ay) = dx]
        //
        // dlen2/dax = -2·dx
        // dlen2/day = -2·dy
        // dlen2/dbx = 2·dx
        // dlen2/dby = 2·dy
        //
        // df/dvar = 2·cross·dcross/dvar - r²·dlen2/dvar

        let dc_dax = -acy + dy;
        let dc_day = -dx + acx;
        let dc_dbx = acy;
        let dc_dby = -acx;
        let dc_dcx = -dy;
        let dc_dcy = dx;

        vec![
            (self.ax, 2.0 * cross * dc_dax - r2 * (-2.0 * dx)),
            (self.ay, 2.0 * cross * dc_day - r2 * (-2.0 * dy)),
            (self.bx, 2.0 * cross * dc_dbx - r2 * (2.0 * dx)),
            (self.by, 2.0 * cross * dc_dby - r2 * (2.0 * dy)),
            (self.cx, 2.0 * cross * dc_dcx),
            (self.cy, 2.0 * cross * dc_dcy),
            (self.r, -2.0 * r * len2),
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

    // --- New equation tests ---

    #[test]
    fn parallel_equation_satisfied() {
        // Line1: (0,0)→(1,0), Line2: (0,1)→(2,1) — both horizontal
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 2.0, 1.0]);
        let eq = ParallelEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn parallel_equation_not_satisfied() {
        // Line1: (0,0)→(1,0), Line2: (0,0)→(0,1) — perpendicular
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let eq = ParallelEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!((eq.eval(&store) - 1.0).abs() < 1e-9); // cross product = 1
    }

    #[test]
    fn collinear_equation_satisfied() {
        // Points (0,0), (1,0), (2,0) — all on x-axis
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 2.0, 0.0]);
        let eq = CollinearEquation::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn collinear_equation_not_satisfied() {
        // Points (0,0), (1,0), (0,1) — triangle
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]);
        let eq = CollinearEquation::new(ids[0], ids[1], ids[2], ids[3], ids[4], ids[5]);
        assert!((eq.eval(&store) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn angle_equation_satisfied() {
        // Line1: (0,0)→(1,0), Line2: (0,0)→(0,1) — 90 degree angle
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let eq = AngleEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
            std::f64::consts::FRAC_PI_2,
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn angle_equation_not_satisfied() {
        // Line1: (0,0)→(1,0), Line2: (0,0)→(1,0) — 0 degree, but target is 90
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
        let eq = AngleEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
            std::f64::consts::FRAC_PI_2,
        );
        assert!(eq.eval(&store).abs() > 0.5); // sin(90)*1 - cos(90)*0 = 1
    }

    #[test]
    fn midpoint_equation_satisfied() {
        // A=0, B=10, Mid=5
        let (store, ids) = make_store(&[0.0, 10.0, 5.0]);
        let eq = MidpointEquation::new(ids[0], ids[1], ids[2]);
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn midpoint_equation_not_satisfied() {
        // A=0, B=10, Mid=3 (should be 5)
        let (store, ids) = make_store(&[0.0, 10.0, 3.0]);
        let eq = MidpointEquation::new(ids[0], ids[1], ids[2]);
        assert!((eq.eval(&store) - (-2.0)).abs() < 1e-9); // 3 - 5 = -2
    }

    #[test]
    fn radius_equation_satisfied() {
        let (store, ids) = make_store(&[5.0]);
        let eq = RadiusEquation::new(ids[0], 5.0);
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn radius_equation_not_satisfied() {
        let (store, ids) = make_store(&[3.0]);
        let eq = RadiusEquation::new(ids[0], 5.0);
        assert_eq!(eq.eval(&store), -2.0);
    }

    #[test]
    fn equal_length_equation_satisfied() {
        // Line1: (0,0)→(3,4) len=5, Line2: (0,0)→(5,0) len=5
        let (store, ids) = make_store(&[0.0, 0.0, 3.0, 4.0, 0.0, 0.0, 5.0, 0.0]);
        let eq = EqualLengthEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn equal_length_equation_not_satisfied() {
        // Line1: (0,0)→(1,0) len=1, Line2: (0,0)→(3,0) len=3
        let (store, ids) = make_store(&[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 3.0, 0.0]);
        let eq = EqualLengthEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!((eq.eval(&store) - (1.0 - 9.0)).abs() < 1e-9); // 1² - 3² = -8
    }

    #[test]
    fn symmetric_perpendicular_satisfied() {
        // P1=(1,0) P2=(-1,0) axis=(0,0)→(0,1) — symmetric about y-axis
        let (store, ids) = make_store(&[1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let eq = SymmetricPerpendicularEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        // (p2x-p1x)*(bx-ax) + (p2y-p1y)*(by-ay) = (-2)*0 + 0*1 = 0
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn symmetric_midpoint_satisfied() {
        // P1=(1,0) P2=(-1,0) axis=(0,0)→(0,1) — midpoint (0,0) is on axis
        let (store, ids) = make_store(&[1.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 1.0]);
        let eq = SymmetricMidpointEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6], ids[7],
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn point_on_circle_satisfied() {
        // Point at (3,4), circle center (0,0), radius 5
        let (store, ids) = make_store(&[3.0, 4.0, 0.0, 0.0, 5.0]);
        let eq = PointOnCircleEquation::new(ids[0], ids[1], ids[2], ids[3], ids[4]);
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn point_on_circle_not_satisfied() {
        // Point at (1,0), circle center (0,0), radius 5
        let (store, ids) = make_store(&[1.0, 0.0, 0.0, 0.0, 5.0]);
        let eq = PointOnCircleEquation::new(ids[0], ids[1], ids[2], ids[3], ids[4]);
        assert!((eq.eval(&store) - (1.0 - 25.0)).abs() < 1e-9); // 1 - 25 = -24
    }

    #[test]
    fn point_on_circle_jacobian() {
        let (store, ids) = make_store(&[3.0, 4.0, 1.0, 1.0, 5.0]);
        let eq = PointOnCircleEquation::new(ids[0], ids[1], ids[2], ids[3], ids[4]);
        let jac = eq.jacobian_row(&store);

        let eps = 1e-7;
        for &(var_id, analytic) in &jac {
            let mut store_plus = VariableStore::new();
            for (i, &v) in [3.0, 4.0, 1.0, 1.0, 5.0].iter().enumerate() {
                let val = if i == var_id.index() as usize { v + eps } else { v };
                store_plus.add(super::super::variable::Variable::new(val));
            }
            let f_plus = eq.eval(&store_plus);
            let f_base = eq.eval(&store);
            let numerical = (f_plus - f_base) / eps;
            assert!(
                (analytic - numerical).abs() < 1e-5,
                "Jacobian mismatch for var {:?}: analytic={}, numerical={}",
                var_id, analytic, numerical
            );
        }
    }

    #[test]
    fn tangent_line_circle_satisfied() {
        // Horizontal line y=0 from (0,0) to (10,0), circle center (5,3) radius 3
        // Distance from (5,3) to the line y=0 is 3 = radius → tangent
        let (store, ids) = make_store(&[0.0, 0.0, 10.0, 0.0, 5.0, 3.0, 3.0]);
        let eq = TangentLineCircleEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6],
        );
        assert!(eq.eval(&store).abs() < 1e-9);
    }

    #[test]
    fn tangent_line_circle_not_satisfied() {
        // Horizontal line y=0, circle center (5,5) radius 3
        // Distance = 5, not 3
        let (store, ids) = make_store(&[0.0, 0.0, 10.0, 0.0, 5.0, 5.0, 3.0]);
        let eq = TangentLineCircleEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6],
        );
        // cross = 10*5 - 0*5 = 50, len2 = 100, r=3
        // f = 50² - 9*100 = 2500 - 900 = 1600
        assert!((eq.eval(&store) - 1600.0).abs() < 1e-6);
    }

    #[test]
    fn tangent_line_circle_jacobian() {
        let vals = [1.0, 2.0, 5.0, 2.0, 3.0, 5.0, 3.0];
        let (store, ids) = make_store(&vals);
        let eq = TangentLineCircleEquation::new(
            ids[0], ids[1], ids[2], ids[3], ids[4], ids[5], ids[6],
        );
        let jac = eq.jacobian_row(&store);

        let eps = 1e-7;
        for &(var_id, analytic) in &jac {
            let mut store_plus = VariableStore::new();
            for (i, &v) in vals.iter().enumerate() {
                let val = if i == var_id.index() as usize { v + eps } else { v };
                store_plus.add(super::super::variable::Variable::new(val));
            }
            let f_plus = eq.eval(&store_plus);
            let f_base = eq.eval(&store);
            let numerical = (f_plus - f_base) / eps;
            assert!(
                (analytic - numerical).abs() < 1e-4,
                "Tangent jacobian mismatch for var {:?}: analytic={}, numerical={}",
                var_id, analytic, numerical
            );
        }
    }
}
