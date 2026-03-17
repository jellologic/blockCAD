use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Surface;

/// A cylindrical surface defined by axis, radius, and reference direction.
/// u parameter is angle [0, 2*PI], v parameter is height along axis.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CylindricalSurface {
    pub origin: Pt3,
    pub axis: Vec3,
    pub ref_dir: Vec3,
    pub radius: f64,
}

impl CylindricalSurface {
    pub fn new(origin: Pt3, axis: Vec3, ref_dir: Vec3, radius: f64) -> KernelResult<Self> {
        if radius <= 0.0 {
            return Err(KernelError::InvalidParameter {
                param: "radius".into(),
                value: radius.to_string(),
            });
        }
        Ok(Self {
            origin,
            axis: axis.normalize(),
            ref_dir: ref_dir.normalize(),
            radius,
        })
    }

    fn binormal(&self) -> Vec3 {
        self.axis.cross(&self.ref_dir)
    }
}

impl Surface for CylindricalSurface {
    fn domain(&self) -> (f64, f64, f64, f64) {
        (0.0, std::f64::consts::TAU, f64::NEG_INFINITY, f64::INFINITY)
    }

    fn point_at(&self, u: f64, v: f64) -> KernelResult<Pt3> {
        let b = self.binormal();
        Ok(self.origin
            + self.radius * (u.cos() * self.ref_dir + u.sin() * b)
            + v * self.axis)
    }

    fn normal_at(&self, u: f64, _v: f64) -> KernelResult<Vec3> {
        let b = self.binormal();
        Ok((u.cos() * self.ref_dir + u.sin() * b).normalize())
    }

    fn derivatives_at(&self, u: f64, _v: f64) -> KernelResult<(Vec3, Vec3)> {
        let b = self.binormal();
        let du = self.radius * (-u.sin() * self.ref_dir + u.cos() * b);
        let dv = self.axis;
        Ok((du, dv))
    }

    fn closest_parameters(&self, _point: &Pt3, _tolerance: f64) -> KernelResult<(f64, f64)> {
        Err(KernelError::Internal(
            "CylindricalSurface::closest_parameters not yet implemented".into(),
        ))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        // Infinite cylinder has no finite bounding box
        Ok(BoundingBox3::new(
            Pt3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
            Pt3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
        ))
    }

    fn is_closed_u(&self) -> bool {
        true
    }

    fn is_closed_v(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Surface> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    fn z_cylinder() -> CylindricalSurface {
        CylindricalSurface::new(
            Pt3::origin(),
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
            2.0,
        )
        .unwrap()
    }

    #[test]
    fn cylinder_point_at_zero() {
        let c = z_cylinder();
        let p = c.point_at(0.0, 0.0).unwrap();
        assert_relative_eq!(p.x, 2.0, epsilon = 1e-9);
        assert_relative_eq!(p.y, 0.0, epsilon = 1e-9);
        assert_relative_eq!(p.z, 0.0, epsilon = 1e-9);
    }

    #[test]
    fn cylinder_normal_is_radial() {
        let c = z_cylinder();
        let n = c.normal_at(0.0, 5.0).unwrap();
        assert_relative_eq!(n, Vec3::new(1.0, 0.0, 0.0), epsilon = 1e-9);
    }

    #[test]
    fn cylinder_is_closed_in_u() {
        let c = z_cylinder();
        assert!(c.is_closed_u());
        assert!(!c.is_closed_v());
    }
}
