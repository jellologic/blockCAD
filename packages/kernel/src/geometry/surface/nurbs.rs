use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Surface;

/// A NURBS surface in 3D, defined by a grid of control points, weights,
/// knot vectors in u and v, and degrees in each direction.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NurbsSurface {
    /// Control points stored in row-major order: [u0v0, u0v1, ..., u0vn, u1v0, ...]
    pub control_points: Vec<Pt3>,
    pub weights: Vec<f64>,
    pub knots_u: Vec<f64>,
    pub knots_v: Vec<f64>,
    pub degree_u: usize,
    pub degree_v: usize,
    /// Number of control points in u direction
    pub num_u: usize,
    /// Number of control points in v direction
    pub num_v: usize,
}

impl NurbsSurface {
    pub fn new(
        control_points: Vec<Pt3>,
        weights: Vec<f64>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
        degree_u: usize,
        degree_v: usize,
        num_u: usize,
        num_v: usize,
    ) -> KernelResult<Self> {
        if control_points.len() != num_u * num_v {
            return Err(KernelError::InvalidParameter {
                param: "control_points".into(),
                value: format!(
                    "expected {}x{}={} points, got {}",
                    num_u,
                    num_v,
                    num_u * num_v,
                    control_points.len()
                ),
            });
        }
        if weights.len() != num_u * num_v {
            return Err(KernelError::InvalidParameter {
                param: "weights".into(),
                value: format!("expected {} weights, got {}", num_u * num_v, weights.len()),
            });
        }
        Ok(Self {
            control_points,
            weights,
            knots_u,
            knots_v,
            degree_u,
            degree_v,
            num_u,
            num_v,
        })
    }
}

impl Surface for NurbsSurface {
    fn domain(&self) -> (f64, f64, f64, f64) {
        let p = self.degree_u;
        let q = self.degree_v;
        (
            self.knots_u[p],
            self.knots_u[self.knots_u.len() - p - 1],
            self.knots_v[q],
            self.knots_v[self.knots_v.len() - q - 1],
        )
    }

    fn point_at(&self, _u: f64, _v: f64) -> KernelResult<Pt3> {
        Err(KernelError::Internal("NurbsSurface::point_at not yet implemented".into()))
    }

    fn normal_at(&self, _u: f64, _v: f64) -> KernelResult<Vec3> {
        Err(KernelError::Internal("NurbsSurface::normal_at not yet implemented".into()))
    }

    fn derivatives_at(&self, _u: f64, _v: f64) -> KernelResult<(Vec3, Vec3)> {
        Err(KernelError::Internal("NurbsSurface::derivatives_at not yet implemented".into()))
    }

    fn closest_parameters(&self, _point: &Pt3, _tolerance: f64) -> KernelResult<(f64, f64)> {
        Err(KernelError::Internal(
            "NurbsSurface::closest_parameters not yet implemented".into(),
        ))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        if self.control_points.is_empty() {
            return Err(KernelError::Geometry("Empty NURBS surface".into()));
        }
        let mut bb = BoundingBox3::from_point(self.control_points[0]);
        for p in &self.control_points[1..] {
            bb.include_point(p);
        }
        Ok(bb)
    }

    fn is_closed_u(&self) -> bool {
        false // TODO: check if first/last row of control points match
    }

    fn is_closed_v(&self) -> bool {
        false // TODO: check if first/last column of control points match
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nurbs_surface_validation() {
        // 2x2 bilinear patch
        let result = NurbsSurface::new(
            vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(1.0, 0.0, 0.0),
                Pt3::new(0.0, 1.0, 0.0),
                Pt3::new(1.0, 1.0, 0.0),
            ],
            vec![1.0; 4],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            1,
            1,
            2,
            2,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn nurbs_surface_wrong_point_count() {
        let result = NurbsSurface::new(
            vec![Pt3::origin(); 3], // should be 4 for 2x2
            vec![1.0; 3],
            vec![0.0, 0.0, 1.0, 1.0],
            vec![0.0, 0.0, 1.0, 1.0],
            1,
            1,
            2,
            2,
        );
        assert!(result.is_err());
    }
}
