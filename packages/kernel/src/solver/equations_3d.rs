//! 3D constraint equations for assembly mate solving.
//!
//! Each component has 6 DOF: tx, ty, tz (translation) + rx, ry, rz (axis-angle rotation).
//! These equations drive the Newton-Raphson solver to find component transforms
//! that satisfy all mate constraints.

use super::equation::Equation;
use super::variable::{VariableId, VariableStore};
use crate::geometry::{Pt3, Vec3, Mat4};
use crate::geometry::transform;

/// Component transform variables: 6 DOF per component.
#[derive(Debug, Clone, Copy)]
pub struct ComponentVars {
    pub tx: VariableId,
    pub ty: VariableId,
    pub tz: VariableId,
    pub rx: VariableId,
    pub ry: VariableId,
    pub rz: VariableId,
}

impl ComponentVars {
    /// Build the 4×4 transform from current variable values.
    pub fn build_transform(&self, vars: &VariableStore) -> Mat4 {
        let tx = vars.value(self.tx);
        let ty = vars.value(self.ty);
        let tz = vars.value(self.tz);
        let rx = vars.value(self.rx);
        let ry = vars.value(self.ry);
        let rz = vars.value(self.rz);

        let axis = Vec3::new(rx, ry, rz);
        let angle = axis.norm();
        let rot = if angle > 1e-12 {
            transform::rotation_axis_angle(&axis, angle)
        } else {
            Mat4::identity()
        };
        transform::compose(&transform::translation(tx, ty, tz), &rot)
    }
}

/// Face geometry in local coordinates (before component transform).
#[derive(Debug, Clone)]
pub struct FaceGeometry {
    /// A point on the face (e.g., centroid)
    pub point: Pt3,
    /// Face normal (outward)
    pub normal: Vec3,
}

/// Axis geometry in local coordinates (for concentric mates).
#[derive(Debug, Clone)]
pub struct AxisGeometry {
    /// A point on the axis
    pub point: Pt3,
    /// Axis direction (unit vector)
    pub direction: Vec3,
}

// ─── COINCIDENT FACE ───────────────────────────────────────────
// Two faces should be coplanar with anti-parallel normals.
// Produces 2 equations:
//   1. Distance between transformed point_a and plane_b = 0
//   2. Normal alignment: dot(n_a, n_b) + 1 = 0 (anti-parallel)

/// Distance from face_a's point to face_b's plane (both transformed).
#[derive(Debug)]
pub struct CoincidentDistanceEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl CoincidentDistanceEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for CoincidentDistanceEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        // Signed distance from p_a to plane (p_b, n_b)
        let diff = p_a - p_b;
        Vec3::new(diff.x, diff.y, diff.z).dot(&n_b)
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        // Numerical differentiation (reliable for 3D transforms)
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Normal alignment: dot(n_a_world, n_b_world) + 1 = 0 (anti-parallel normals).
#[derive(Debug)]
pub struct CoincidentNormalEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl CoincidentNormalEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for CoincidentNormalEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let n_a = transform::transform_normal(&t_a, &self.face_a.normal);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        // For coincident: normals should be anti-parallel → dot = -1
        n_a.dot(&n_b) + 1.0
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── CONCENTRIC ────────────────────────────────────────────────
// Two cylindrical axes should be collinear.
// Produces 2 equations:
//   1. Perpendicular distance between axes = 0
//   2. Direction alignment: |cross(dir_a, dir_b)| = 0

/// Perpendicular distance between two axes.
#[derive(Debug)]
pub struct ConcentricDistanceEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub axis_a: AxisGeometry,
    pub axis_b: AxisGeometry,
    ids: Vec<VariableId>,
}

impl ConcentricDistanceEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, axis_a: AxisGeometry, axis_b: AxisGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, axis_a, axis_b, ids }
    }
}

impl Equation for ConcentricDistanceEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.axis_a.point);
        let p_b = transform::transform_point(&t_b, &self.axis_b.point);
        let d_b = transform::transform_normal(&t_b, &self.axis_b.direction);
        // Distance from p_a to the line (p_b, d_b):
        // Project (p_a - p_b) onto plane perpendicular to d_b
        let diff = p_a - p_b;
        let diff_vec = Vec3::new(diff.x, diff.y, diff.z);
        let along = diff_vec.dot(&d_b);
        let perp = diff_vec - d_b * along;
        perp.norm()
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

/// Axis direction alignment: |cross(dir_a, dir_b)| = 0.
#[derive(Debug)]
pub struct ConcentricAlignEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub axis_a: AxisGeometry,
    pub axis_b: AxisGeometry,
    ids: Vec<VariableId>,
}

impl ConcentricAlignEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, axis_a: AxisGeometry, axis_b: AxisGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, axis_a, axis_b, ids }
    }
}

impl Equation for ConcentricAlignEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let d_a = transform::transform_normal(&t_a, &self.axis_a.direction);
        let d_b = transform::transform_normal(&t_b, &self.axis_b.direction);
        d_a.cross(&d_b).norm()
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── DISTANCE MATE ─────────────────────────────────────────────
// Face-to-face distance equals a target value.

#[derive(Debug)]
pub struct DistanceMateEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    pub target_distance: f64,
    ids: Vec<VariableId>,
}

impl DistanceMateEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry, target: f64) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, target_distance: target, ids }
    }
}

impl Equation for DistanceMateEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        let diff = p_a - p_b;
        let signed_dist = Vec3::new(diff.x, diff.y, diff.z).dot(&n_b);
        // Use signed distance — positive means face_a is on normal side of face_b
        signed_dist - self.target_distance
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── ANGLE MATE ────────────────────────────────────────────────
// Angle between two face normals equals a target value.

#[derive(Debug)]
pub struct AngleMateEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    pub target_angle: f64,
    ids: Vec<VariableId>,
}

impl AngleMateEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry, target: f64) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, target_angle: target, ids }
    }
}

impl Equation for AngleMateEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let n_a = transform::transform_normal(&t_a, &self.face_a.normal);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        let cos_angle = n_a.dot(&n_b).clamp(-1.0, 1.0);
        cos_angle.acos() - self.target_angle
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── PARALLEL ──────────────────────────────────────────────────
// Face normals should be parallel: |cross(n_a, n_b)| = 0

#[derive(Debug)]
pub struct ParallelEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl ParallelEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for ParallelEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let n_a = transform::transform_normal(&t_a, &self.face_a.normal);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        n_a.cross(&n_b).norm()
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── PERPENDICULAR ─────────────────────────────────────────────
// Face normals should be perpendicular: dot(n_a, n_b) = 0

#[derive(Debug)]
pub struct PerpendicularEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl PerpendicularEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for PerpendicularEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let n_a = transform::transform_normal(&t_a, &self.face_a.normal);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        n_a.dot(&n_b)
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── TANGENT ───────────────────────────────────────────────────
// Faces touch tangentially: distance between face points = 0 (surfaces touch)

#[derive(Debug)]
pub struct TangentEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl TangentEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for TangentEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        // Signed distance from face_a centroid to face_b plane = 0 (touching)
        let diff = p_a - p_b;
        Vec3::new(diff.x, diff.y, diff.z).dot(&n_b)
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── HINGE ─────────────────────────────────────────────────────
// Hinge = concentric (axis collinear) — same equations as Concentric.
// The solver naturally allows rotation around the shared axis since
// only axis alignment + point proximity are constrained, not the rotation angle.
// Reuse ConcentricDistanceEquation + ConcentricAlignEquation.

// ─── GEAR ──────────────────────────────────────────────────────
// Coupled rotation: rx_a * ratio = rx_b

#[derive(Debug)]
pub struct GearEquation {
    pub rx_a: VariableId,
    pub rx_b: VariableId,
    pub ratio: f64,
    ids: Vec<VariableId>,
}

impl GearEquation {
    pub fn new(rx_a: VariableId, rx_b: VariableId, ratio: f64) -> Self {
        Self { rx_a, rx_b, ratio, ids: vec![rx_a, rx_b] }
    }
}

impl Equation for GearEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        vars.value(self.rx_a) * self.ratio - vars.value(self.rx_b)
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![(self.rx_a, self.ratio), (self.rx_b, -1.0)]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── SCREW ─────────────────────────────────────────────────────
// Linear↔rotational coupling: tz_b = rx_b * pitch / (2π)

#[derive(Debug)]
pub struct ScrewEquation {
    pub tz: VariableId,
    pub rx: VariableId,
    pub pitch: f64,
    ids: Vec<VariableId>,
}

impl ScrewEquation {
    pub fn new(tz: VariableId, rx: VariableId, pitch: f64) -> Self {
        Self { tz, rx, pitch, ids: vec![tz, rx] }
    }
}

impl Equation for ScrewEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let expected_z = vars.value(self.rx) * self.pitch / (2.0 * std::f64::consts::PI);
        vars.value(self.tz) - expected_z
    }

    fn jacobian_row(&self, _vars: &VariableStore) -> Vec<(VariableId, f64)> {
        vec![
            (self.tz, 1.0),
            (self.rx, -self.pitch / (2.0 * std::f64::consts::PI)),
        ]
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── LIMIT ─────────────────────────────────────────────────────
// Soft clamp: penalty when face distance is outside [min, max].
// f = max(0, min_val - dist) + max(0, dist - max_val)

#[derive(Debug)]
pub struct LimitEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    pub min_val: f64,
    pub max_val: f64,
    ids: Vec<VariableId>,
}

impl LimitEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry, min_val: f64, max_val: f64) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, min_val, max_val, ids }
    }
}

impl Equation for LimitEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        let diff = p_a - p_b;
        let dist = Vec3::new(diff.x, diff.y, diff.z).dot(&n_b);
        // Penalty: 0 when within [min, max], positive when outside
        if dist < self.min_val {
            self.min_val - dist
        } else if dist > self.max_val {
            dist - self.max_val
        } else {
            0.0
        }
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── WIDTH ─────────────────────────────────────────────────────
// Center comp_b midway between face_a and face_b along face_b's normal.
// midpoint = (p_a + p_b) / 2 along normal_b direction
// constraint: dot(p_comp_b - midpoint, n_b) = 0

#[derive(Debug)]
pub struct WidthEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl WidthEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for WidthEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        let n_b = transform::transform_normal(&t_b, &self.face_b.normal);
        // Midpoint between the two face points
        let mid = Pt3::new((p_a.x + p_b.x) / 2.0, (p_a.y + p_b.y) / 2.0, (p_a.z + p_b.z) / 2.0);
        // comp_b center should be at this midpoint (along normal direction)
        let comp_b_center = transform::transform_point(&t_b, &Pt3::origin());
        let diff = comp_b_center - mid;
        Vec3::new(diff.x, diff.y, diff.z).dot(&n_b)
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── SYMMETRIC ─────────────────────────────────────────────────
// Two face points symmetric across a midplane.
// face_a and face_b should be equidistant from the origin plane (along normal).
// constraint: dot(p_a + p_b, n) / 2 - dot(origin, n) = 0

#[derive(Debug)]
pub struct SymmetricEquation {
    pub comp_a: ComponentVars,
    pub comp_b: ComponentVars,
    pub face_a: FaceGeometry,
    pub face_b: FaceGeometry,
    ids: Vec<VariableId>,
}

impl SymmetricEquation {
    pub fn new(comp_a: ComponentVars, comp_b: ComponentVars, face_a: FaceGeometry, face_b: FaceGeometry) -> Self {
        let ids = vec![comp_a.tx, comp_a.ty, comp_a.tz, comp_a.rx, comp_a.ry, comp_a.rz,
                       comp_b.tx, comp_b.ty, comp_b.tz, comp_b.rx, comp_b.ry, comp_b.rz];
        Self { comp_a, comp_b, face_a, face_b, ids }
    }
}

impl Equation for SymmetricEquation {
    fn eval(&self, vars: &VariableStore) -> f64 {
        let t_a = self.comp_a.build_transform(vars);
        let t_b = self.comp_b.build_transform(vars);
        let p_a = transform::transform_point(&t_a, &self.face_a.point);
        let p_b = transform::transform_point(&t_b, &self.face_b.point);
        // Use face_a's normal as the symmetry direction
        let n = transform::transform_normal(&t_a, &self.face_a.normal);
        // Signed distances from origin along normal
        let d_a = Vec3::new(p_a.x, p_a.y, p_a.z).dot(&n);
        let d_b = Vec3::new(p_b.x, p_b.y, p_b.z).dot(&n);
        // Symmetric: d_a + d_b = 0 (equidistant from origin along normal)
        d_a + d_b
    }

    fn jacobian_row(&self, vars: &VariableStore) -> Vec<(VariableId, f64)> {
        numerical_jacobian(self, vars, &self.ids)
    }

    fn variable_ids(&self) -> &[VariableId] {
        &self.ids
    }
}

// ─── LOCK ──────────────────────────────────────────────────────
// Lock all 6 DOF of a component at its current position.
// Produces 6 equations: tx=T, ty=T, tz=T, rx=0, ry=0, rz=0

#[derive(Debug)]
pub struct LockEquation {
    pub var: VariableId,
    pub target: f64,
    ids: Vec<VariableId>,
}

impl LockEquation {
    pub fn new(var: VariableId, target: f64) -> Self {
        Self { var, target, ids: vec![var] }
    }
}

impl Equation for LockEquation {
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

// ─── NUMERICAL JACOBIAN ────────────────────────────────────────

/// Compute Jacobian row via central differences (robust for 3D transforms).
fn numerical_jacobian(eq: &dyn Equation, vars: &VariableStore, ids: &[VariableId]) -> Vec<(VariableId, f64)> {
    let h = 1e-7;
    let mut result = Vec::new();
    // Clone variable values for perturbation
    let mut vars_plus = vars.clone();
    let mut vars_minus = vars.clone();

    for &var_id in ids {
        let orig = vars.value(var_id);
        vars_plus.set_value(var_id, orig + h);
        vars_minus.set_value(var_id, orig - h);
        let f_plus = eq.eval(&vars_plus);
        let f_minus = eq.eval(&vars_minus);
        let deriv = (f_plus - f_minus) / (2.0 * h);
        if deriv.abs() > 1e-14 {
            result.push((var_id, deriv));
        }
        // Reset
        vars_plus.set_value(var_id, orig);
        vars_minus.set_value(var_id, orig);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::solver::variable::Variable;

    fn make_grounded_vars(vars: &mut VariableStore) -> ComponentVars {
        ComponentVars {
            tx: vars.add(Variable::fixed(0.0)),
            ty: vars.add(Variable::fixed(0.0)),
            tz: vars.add(Variable::fixed(0.0)),
            rx: vars.add(Variable::fixed(0.0)),
            ry: vars.add(Variable::fixed(0.0)),
            rz: vars.add(Variable::fixed(0.0)),
        }
    }

    fn make_free_vars(vars: &mut VariableStore, tx: f64, ty: f64, tz: f64) -> ComponentVars {
        ComponentVars {
            tx: vars.add(Variable::new(tx)),
            ty: vars.add(Variable::new(ty)),
            tz: vars.add(Variable::new(tz)),
            rx: vars.add(Variable::new(0.0)),
            ry: vars.add(Variable::new(0.0)),
            rz: vars.add(Variable::new(0.0)),
        }
    }

    #[test]
    fn coincident_distance_zero_when_coplanar() {
        let mut vars = VariableStore::new();
        let comp_a = make_grounded_vars(&mut vars);
        let comp_b = make_free_vars(&mut vars, 0.0, 0.0, 0.0);

        let face_a = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, 1.0) };
        let face_b = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, -1.0) };

        let eq = CoincidentDistanceEquation::new(comp_a, comp_b, face_a, face_b);
        assert!(eq.eval(&vars).abs() < 1e-9);
    }

    #[test]
    fn coincident_distance_nonzero_when_offset() {
        let mut vars = VariableStore::new();
        let comp_a = make_grounded_vars(&mut vars);
        let comp_b = make_free_vars(&mut vars, 0.0, 0.0, 3.0); // offset by 3 in Z

        let face_a = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, 1.0) };
        let face_b = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, -1.0) };

        let eq = CoincidentDistanceEquation::new(comp_a, comp_b, face_a, face_b);
        // comp_b face point is at z=5+3=8, comp_a face is at z=5, normal_b is -Z
        // distance = (5-8) · (0,0,-1) = 3
        assert!((eq.eval(&vars) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn coincident_normal_zero_when_antiparallel() {
        let mut vars = VariableStore::new();
        let comp_a = make_grounded_vars(&mut vars);
        let comp_b = make_free_vars(&mut vars, 0.0, 0.0, 0.0);

        let face_a = FaceGeometry { point: Pt3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) };
        let face_b = FaceGeometry { point: Pt3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) };

        let eq = CoincidentNormalEquation::new(comp_a, comp_b, face_a, face_b);
        assert!(eq.eval(&vars).abs() < 1e-9);
    }

    #[test]
    fn distance_mate_satisfied() {
        let mut vars = VariableStore::new();
        let comp_a = make_grounded_vars(&mut vars);
        let comp_b = make_free_vars(&mut vars, 0.0, 0.0, 5.0);

        let face_a = FaceGeometry { point: Pt3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, 1.0) };
        let face_b = FaceGeometry { point: Pt3::new(0.0, 0.0, 0.0), normal: Vec3::new(0.0, 0.0, -1.0) };

        let eq = DistanceMateEquation::new(comp_a, comp_b, face_a, face_b, 5.0);
        assert!(eq.eval(&vars).abs() < 1e-9);
    }

    #[test]
    fn numerical_jacobian_produces_values() {
        let mut vars = VariableStore::new();
        let comp_a = make_grounded_vars(&mut vars);
        let comp_b = make_free_vars(&mut vars, 0.0, 0.0, 1.0);

        let face_a = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, 1.0) };
        let face_b = FaceGeometry { point: Pt3::new(0.0, 0.0, 5.0), normal: Vec3::new(0.0, 0.0, -1.0) };

        let eq = CoincidentDistanceEquation::new(comp_a, comp_b, face_a, face_b);
        let jac = eq.jacobian_row(&vars);
        // Should have non-zero derivatives w.r.t. comp_b's tz
        assert!(!jac.is_empty(), "Jacobian should have non-zero entries");
    }
}
