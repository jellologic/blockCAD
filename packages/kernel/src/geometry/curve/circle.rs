use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Curve;

/// A full circle in 3D space. Parameterized t in [0, 1] mapping to [0, 2*PI].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Circle3 {
    pub center: Pt3,
    pub radius: f64,
    pub normal: Vec3,
    pub ref_dir: Vec3,
}

impl Circle3 {
    pub fn new(center: Pt3, radius: f64, normal: Vec3, ref_dir: Vec3) -> KernelResult<Self> {
        if radius <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "radius".into(),
                value: radius.to_string(),
            });
        }
        Ok(Self {
            center,
            radius,
            normal: normal.normalize(),
            ref_dir: ref_dir.normalize(),
        })
    }

    fn binormal(&self) -> Vec3 {
        self.normal.cross(&self.ref_dir)
    }
}

impl Curve for Circle3 {
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn point_at(&self, t: f64) -> KernelResult<Pt3> {
        let angle = t * std::f64::consts::TAU;
        let b = self.binormal();
        Ok(self.center + self.radius * (angle.cos() * self.ref_dir + angle.sin() * b))
    }

    fn tangent_at(&self, t: f64) -> KernelResult<Vec3> {
        let angle = t * std::f64::consts::TAU;
        let b = self.binormal();
        Ok(self.radius * std::f64::consts::TAU * (-angle.sin() * self.ref_dir + angle.cos() * b))
    }

    fn second_derivative_at(&self, t: f64) -> KernelResult<Vec3> {
        let angle = t * std::f64::consts::TAU;
        let b = self.binormal();
        let tau_sq = std::f64::consts::TAU * std::f64::consts::TAU;
        Ok(self.radius * tau_sq * (-angle.cos() * self.ref_dir - angle.sin() * b))
    }

    fn arc_length(&self, t0: f64, t1: f64, _tolerance: f64) -> KernelResult<f64> {
        Ok(self.radius * std::f64::consts::TAU * (t1 - t0).abs())
    }

    fn closest_parameter(&self, _point: &Pt3, _tolerance: f64) -> KernelResult<f64> {
        Err(KernelError::Internal("Circle3::closest_parameter not yet implemented".into()))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        let b = self.binormal();
        let mut bb = BoundingBox3::from_point(self.center + self.radius * self.ref_dir);
        bb.include_point(&(self.center - self.radius * self.ref_dir));
        bb.include_point(&(self.center + self.radius * b));
        bb.include_point(&(self.center - self.radius * b));
        Ok(bb)
    }

    fn is_closed(&self) -> bool {
        true
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
    fn circle_is_closed() {
        let c = Circle3::new(
            Pt3::origin(),
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        assert!(c.is_closed());
    }

    #[test]
    fn circle_circumference() {
        let c = Circle3::new(
            Pt3::origin(),
            2.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let len = c.arc_length(0.0, 1.0, 1e-9).unwrap();
        assert_relative_eq!(len, 2.0 * std::f64::consts::TAU, epsilon = 1e-9);
    }

    #[test]
    fn circle_start_equals_end() {
        let c = Circle3::new(
            Pt3::origin(),
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let start = c.point_at(0.0).unwrap();
        let end = c.point_at(1.0).unwrap();
        assert_relative_eq!(start.x, end.x, epsilon = 1e-9);
        assert_relative_eq!(start.y, end.y, epsilon = 1e-9);
    }
}
