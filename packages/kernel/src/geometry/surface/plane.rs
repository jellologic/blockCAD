use crate::error::KernelResult;
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Surface;

/// An infinite plane defined by an origin point, normal, and two in-plane axes.
/// Parameterized with u and v as distances along the axes.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Plane {
    pub origin: Pt3,
    pub normal: Vec3,
    pub u_axis: Vec3,
    pub v_axis: Vec3,
}

impl Plane {
    /// Create a plane from origin and normal. Axes are computed automatically.
    pub fn from_normal(origin: Pt3, normal: Vec3) -> Self {
        let n = normal.normalize();
        // Pick a reference vector not parallel to normal
        let reference = if n.x.abs() < 0.9 {
            Vec3::new(1.0, 0.0, 0.0)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        };
        let u_axis = n.cross(&reference).normalize();
        let v_axis = n.cross(&u_axis).normalize();
        Self {
            origin,
            normal: n,
            u_axis,
            v_axis,
        }
    }

    /// XY plane at a given Z height
    pub fn xy(z: f64) -> Self {
        Self {
            origin: Pt3::new(0.0, 0.0, z),
            normal: Vec3::new(0.0, 0.0, 1.0),
            u_axis: Vec3::new(1.0, 0.0, 0.0),
            v_axis: Vec3::new(0.0, 1.0, 0.0),
        }
    }

    /// Distance from a point to this plane (signed)
    pub fn signed_distance(&self, point: &Pt3) -> f64 {
        (point - self.origin).dot(&self.normal)
    }
}

impl Surface for Plane {
    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(self.clone())
    }

    fn domain(&self) -> (f64, f64, f64, f64) {
        (f64::NEG_INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::INFINITY)
    }

    #[inline]
    fn point_at(&self, u: f64, v: f64) -> KernelResult<Pt3> {
        Ok(self.origin + u * self.u_axis + v * self.v_axis)
    }

    #[inline]
    fn normal_at(&self, _u: f64, _v: f64) -> KernelResult<Vec3> {
        Ok(self.normal)
    }

    fn derivatives_at(&self, _u: f64, _v: f64) -> KernelResult<(Vec3, Vec3)> {
        Ok((self.u_axis, self.v_axis))
    }

    #[inline]
    fn closest_parameters(&self, point: &Pt3, _tolerance: f64) -> KernelResult<(f64, f64)> {
        let v = point - self.origin;
        Ok((v.dot(&self.u_axis), v.dot(&self.v_axis)))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        // An infinite plane has no finite bounding box
        Ok(BoundingBox3::new(
            Pt3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            Pt3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        ))
    }

    fn is_closed_u(&self) -> bool {
        false
    }

    fn is_closed_v(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn plane_point_at() {
        let p = Plane::xy(0.0);
        let pt = p.point_at(3.0, 4.0).unwrap();
        assert_relative_eq!(pt.x, 3.0, epsilon = 1e-9);
        assert_relative_eq!(pt.y, 4.0, epsilon = 1e-9);
        assert_relative_eq!(pt.z, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn plane_normal_constant() {
        let p = Plane::xy(5.0);
        let n1 = p.normal_at(0.0, 0.0).unwrap();
        let n2 = p.normal_at(100.0, -50.0).unwrap();
        assert_relative_eq!(n1, n2, epsilon = 1e-9);
        assert_relative_eq!(n1, Vec3::new(0.0, 0.0, 1.0), epsilon = 1e-9);
    }

    #[test]
    fn plane_closest_parameters() {
        let p = Plane::xy(0.0);
        let (u, v) = p.closest_parameters(&Pt3::new(7.0, 3.0, 10.0), 1e-9).unwrap();
        assert_relative_eq!(u, 7.0, epsilon = 1e-9);
        assert_relative_eq!(v, 3.0, epsilon = 1e-9);
    }

    #[test]
    fn plane_signed_distance() {
        let p = Plane::xy(0.0);
        assert_relative_eq!(p.signed_distance(&Pt3::new(0.0, 0.0, 5.0)), 5.0, epsilon = 1e-9);
        assert_relative_eq!(p.signed_distance(&Pt3::new(0.0, 0.0, -3.0)), -3.0, epsilon = 1e-9);
    }
}
