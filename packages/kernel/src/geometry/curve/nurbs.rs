use crate::error::{KernelError, KernelResult};
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

use super::Curve;

/// A NURBS (Non-Uniform Rational B-Spline) curve in 3D.
///
/// Defined by control points, weights, knot vector, and degree.
/// This is the most general curve representation — lines, arcs, conics,
/// and free-form curves can all be expressed as NURBS.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NurbsCurve {
    pub control_points: Vec<Pt3>,
    pub weights: Vec<f64>,
    pub knots: Vec<f64>,
    pub degree: usize,
}

impl NurbsCurve {
    pub fn new(
        control_points: Vec<Pt3>,
        weights: Vec<f64>,
        knots: Vec<f64>,
        degree: usize,
    ) -> KernelResult<Self> {
        let n = control_points.len();
        if n < degree + 1 {
            return Err(KernelError::InvalidParameter {
                param: "control_points".into(),
                value: format!("need at least {} points for degree {}", degree + 1, degree),
            });
        }
        if weights.len() != n {
            return Err(KernelError::InvalidParameter {
                param: "weights".into(),
                value: format!("expected {} weights, got {}", n, weights.len()),
            });
        }
        let expected_knots = n + degree + 1;
        if knots.len() != expected_knots {
            return Err(KernelError::InvalidParameter {
                param: "knots".into(),
                value: format!("expected {} knots, got {}", expected_knots, knots.len()),
            });
        }
        Ok(Self {
            control_points,
            weights,
            knots,
            degree,
        })
    }
}

impl Curve for NurbsCurve {
    fn domain(&self) -> (f64, f64) {
        let p = self.degree;
        (self.knots[p], self.knots[self.knots.len() - p - 1])
    }

    fn point_at(&self, _t: f64) -> KernelResult<Pt3> {
        // TODO: implement Cox-de Boor evaluation
        Err(KernelError::Internal("NurbsCurve::point_at not yet implemented".into()))
    }

    fn tangent_at(&self, _t: f64) -> KernelResult<Vec3> {
        Err(KernelError::Internal("NurbsCurve::tangent_at not yet implemented".into()))
    }

    fn second_derivative_at(&self, _t: f64) -> KernelResult<Vec3> {
        Err(KernelError::Internal(
            "NurbsCurve::second_derivative_at not yet implemented".into(),
        ))
    }

    fn arc_length(&self, _t0: f64, _t1: f64, _tolerance: f64) -> KernelResult<f64> {
        Err(KernelError::Internal("NurbsCurve::arc_length not yet implemented".into()))
    }

    fn closest_parameter(&self, _point: &Pt3, _tolerance: f64) -> KernelResult<f64> {
        Err(KernelError::Internal(
            "NurbsCurve::closest_parameter not yet implemented".into(),
        ))
    }

    fn bounding_box(&self) -> KernelResult<BoundingBox3> {
        if self.control_points.is_empty() {
            return Err(KernelError::Geometry("Empty NURBS curve".into()));
        }
        let mut bb = BoundingBox3::from_point(self.control_points[0]);
        for p in &self.control_points[1..] {
            bb.include_point(p);
        }
        Ok(bb)
    }

    fn is_closed(&self) -> bool {
        if self.control_points.len() < 2 {
            return false;
        }
        let first = &self.control_points[0];
        let last = &self.control_points[self.control_points.len() - 1];
        (last - first).norm() < crate::geometry::GEOMETRIC_TOLERANCE
    }

    fn clone_box(&self) -> Box<dyn Curve> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nurbs_validation() {
        // Valid cubic NURBS with 4 control points
        let result = NurbsCurve::new(
            vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(1.0, 1.0, 0.0),
                Pt3::new(2.0, 1.0, 0.0),
                Pt3::new(3.0, 0.0, 0.0),
            ],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            3,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn nurbs_too_few_points() {
        let result = NurbsCurve::new(
            vec![Pt3::origin(), Pt3::new(1.0, 0.0, 0.0)],
            vec![1.0, 1.0],
            vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
            3, // degree 3 needs at least 4 points
        );
        assert!(result.is_err());
    }

    #[test]
    fn nurbs_domain() {
        let c = NurbsCurve::new(
            vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(1.0, 1.0, 0.0),
                Pt3::new(2.0, 1.0, 0.0),
                Pt3::new(3.0, 0.0, 0.0),
            ],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            3,
        )
        .unwrap();
        assert_eq!(c.domain(), (0.0, 1.0));
    }

    #[test]
    fn nurbs_bounding_box_from_control_points() {
        let c = NurbsCurve::new(
            vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(1.0, 2.0, 0.0),
                Pt3::new(2.0, -1.0, 0.0),
                Pt3::new(3.0, 0.0, 0.0),
            ],
            vec![1.0, 1.0, 1.0, 1.0],
            vec![0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0],
            3,
        )
        .unwrap();
        let bb = c.bounding_box().unwrap();
        assert_eq!(bb.min, Pt3::new(0.0, -1.0, 0.0));
        assert_eq!(bb.max, Pt3::new(3.0, 2.0, 0.0));
    }
}
