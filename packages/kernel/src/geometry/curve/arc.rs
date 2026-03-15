use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Curve;

/// A circular arc in 3D space, defined by center, radius, normal, and angular range.
/// Parameterized t in [0, 1] mapping to [start_angle, end_angle].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Arc3 {
    pub center: Pt3,
    pub radius: f64,
    /// Unit normal to the arc plane
    pub normal: Vec3,
    /// Reference direction in the arc plane (unit vector from center to start point)
    pub ref_dir: Vec3,
    /// Start angle in radians
    pub start_angle: f64,
    /// End angle in radians (must be > start_angle)
    pub end_angle: f64,
}

impl Arc3 {
    pub fn new(
        center: Pt3,
        radius: f64,
        normal: Vec3,
        ref_dir: Vec3,
        start_angle: f64,
        end_angle: f64,
    ) -> KernelResult<Self> {
        if radius <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "radius".into(),
                value: radius.to_string(),
            });
        }
        if (end_angle - start_angle).abs() < crate::geometry::ANGLE_TOLERANCE {
            return Err(KernelError::Geometry("Degenerate arc: zero sweep angle".into()));
        }
        Ok(Self {
            center,
            radius,
            normal: normal.normalize(),
            ref_dir: ref_dir.normalize(),
            start_angle,
            end_angle,
        })
    }

    fn angle_at(&self, t: f64) -> f64 {
        self.start_angle + t * (self.end_angle - self.start_angle)
    }

    fn binormal(&self) -> Vec3 {
        self.normal.cross(&self.ref_dir)
    }
}

impl Curve for Arc3 {
    fn domain(&self) -> (f64, f64) {
        (0.0, 1.0)
    }

    fn point_at(&self, t: f64) -> KernelResult<Pt3> {
        let angle = self.angle_at(t);
        let b = self.binormal();
        Ok(self.center + self.radius * (angle.cos() * self.ref_dir + angle.sin() * b))
    }

    fn tangent_at(&self, t: f64) -> KernelResult<Vec3> {
        let angle = self.angle_at(t);
        let b = self.binormal();
        let sweep = self.end_angle - self.start_angle;
        Ok(self.radius * sweep * (-angle.sin() * self.ref_dir + angle.cos() * b))
    }

    fn second_derivative_at(&self, t: f64) -> KernelResult<Vec3> {
        let angle = self.angle_at(t);
        let b = self.binormal();
        let sweep = self.end_angle - self.start_angle;
        Ok(self.radius * sweep * sweep * (-angle.cos() * self.ref_dir - angle.sin() * b))
    }

    fn arc_length(&self, t0: f64, t1: f64, _tolerance: f64) -> KernelResult<f64> {
        let sweep = (self.end_angle - self.start_angle).abs();
        Ok(self.radius * sweep * (t1 - t0).abs())
    }

    fn closest_parameter(&self, _point: &Pt3, _tolerance: f64) -> KernelResult<f64> {
        // TODO: implement proper inverse evaluation
        Err(KernelError::Internal("Arc3::closest_parameter not yet implemented".into()))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        // Conservative: sample several points
        let n = 32;
        let start = self.point_at(0.0)?;
        let mut bb = BoundingBox3::from_point(start);
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let p = self.point_at(t)?;
            bb.include_point(&p);
        }
        Ok(bb)
    }

    fn is_closed(&self) -> bool {
        ((self.end_angle - self.start_angle).abs() - std::f64::consts::TAU).abs()
            < crate::geometry::ANGLE_TOLERANCE
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;
    use std::f64::consts::PI;

    fn xy_arc() -> Arc3 {
        Arc3::new(
            Pt3::origin(),
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            0.0,
            PI / 2.0,
        )
        .unwrap()
    }

    #[test]
    fn arc_start_point() {
        let arc = xy_arc();
        let p = arc.point_at(0.0).unwrap();
        assert_relative_eq!(p.x, 1.0, epsilon = 1e-9);
        assert_relative_eq!(p.y, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn arc_end_point() {
        let arc = xy_arc();
        let p = arc.point_at(1.0).unwrap();
        assert_relative_eq!(p.x, 0.0, epsilon = 1e-9);
        assert_relative_eq!(p.y, 1.0, epsilon = 1e-9);
    }

    #[test]
    fn arc_length_quarter_circle() {
        let arc = xy_arc();
        let len = arc.arc_length(0.0, 1.0, 1e-9).unwrap();
        assert_relative_eq!(len, PI / 2.0, epsilon = 1e-9);
    }

    #[test]
    fn invalid_radius_rejected() {
        let result = Arc3::new(
            Pt3::origin(),
            -1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            0.0,
            PI,
        );
        assert!(result.is_err());
    }
}
