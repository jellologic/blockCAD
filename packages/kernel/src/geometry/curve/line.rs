use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Curve;

/// An infinite line segment defined by start and end points, parameterized t in [0, 1].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Line3 {
    pub start: Pt3,
    pub end: Pt3,
}

impl Line3 {
    pub fn new(start: Pt3, end: Pt3) -> KernelResult<Self> {
        let diff = end - start;
        if diff.norm() < crate::geometry::GEOMETRIC_TOLERANCE {
            return Err(KernelError::Geometry("Degenerate line: start equals end".into()));
        }
        Ok(Self { start, end })
    }

    pub fn direction(&self) -> Vec3 {
        (self.end - self.start).normalize()
    }

    pub fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }
}

impl Curve for Line3 {
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn point_at(&self, t: f64) -> KernelResult<Pt3> {
        Ok(self.start + t * (self.end - self.start))
    }

    fn tangent_at(&self, _t: f64) -> KernelResult<Vec3> {
        Ok(self.end - self.start)
    }

    fn second_derivative_at(&self, _t: f64) -> KernelResult<Vec3> {
        Ok(Vec3::zeros())
    }

    fn arc_length(&self, t0: f64, t1: f64, _tolerance: f64) -> KernelResult<f64> {
        Ok((t1 - t0).abs() * self.length())
    }

    fn closest_parameter(&self, point: &Pt3, _tolerance: f64) -> KernelResult<f64> {
        let d = self.end - self.start;
        let t = (point - self.start).dot(&d) / d.norm_squared();
        Ok(t.clamp(0.0, 1.0))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        let mut bb = BoundingBox3::from_point(self.start);
        bb.include_point(&self.end);
        Ok(bb)
    }

    fn is_closed(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn line_point_at() {
        let line = Line3::new(Pt3::origin(), Pt3::new(2.0, 0.0, 0.0)).unwrap();
        let mid = line.point_at(0.5).unwrap();
        assert_relative_eq!(mid.x, 1.0, epsilon = 1e-9);
        assert_relative_eq!(mid.y, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn line_tangent_is_constant() {
        let line = Line3::new(Pt3::origin(), Pt3::new(1.0, 1.0, 0.0)).unwrap();
        let t0 = line.tangent_at(0.0).unwrap();
        let t1 = line.tangent_at(1.0).unwrap();
        assert_relative_eq!(t0, t1, epsilon = 1e-9);
    }

    #[test]
    fn line_arc_length() {
        let line = Line3::new(Pt3::origin(), Pt3::new(3.0, 4.0, 0.0)).unwrap();
        let len = line.arc_length(0.0, 1.0, 1e-9).unwrap();
        assert_relative_eq!(len, 5.0, epsilon = 1e-9);
    }

    #[test]
    fn line_closest_parameter() {
        let line = Line3::new(Pt3::origin(), Pt3::new(10.0, 0.0, 0.0)).unwrap();
        let t = line.closest_parameter(&Pt3::new(5.0, 3.0, 0.0), 1e-9).unwrap();
        assert_relative_eq!(t, 0.5, epsilon = 1e-9);
    }

    #[test]
    fn degenerate_line_rejected() {
        let result = Line3::new(Pt3::origin(), Pt3::origin());
        assert!(result.is_err());
    }
}
