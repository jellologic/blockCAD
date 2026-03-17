pub mod arc;
pub mod circle;
pub mod line;
pub mod nurbs;

use crate::error::KernelResult;
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

/// Trait for 3D parametric curves.
///
/// All curves are parameterized over a domain [t_min, t_max].
/// Implementations must be thread-safe for parallel tessellation.
pub trait Curve: Send + Sync + std::fmt::Debug {
    /// Parameter domain [t_min, t_max]
    fn domain(&self) -> (f64, f64);

    /// Evaluate point on curve at parameter t
    fn point_at(&self, t: f64) -> KernelResult<Pt3>;

    /// First derivative (tangent vector) at parameter t
    fn tangent_at(&self, t: f64) -> KernelResult<Vec3>;

    /// Second derivative at parameter t (for curvature computation)
    fn second_derivative_at(&self, t: f64) -> KernelResult<Vec3>;

    /// Approximate arc length between parameters t0 and t1
    fn arc_length(&self, t0: f64, t1: f64, tolerance: f64) -> KernelResult<f64>;

    /// Find the closest parameter value to a given point
    fn closest_parameter(&self, point: &Pt3, tolerance: f64) -> KernelResult<f64>;

    /// Compute axis-aligned bounding box
    fn bounding_box(&self) -> KernelResult<BoundingBox3>;

    /// Whether the curve is geometrically closed
    fn is_closed(&self) -> bool;

    /// Clone this curve into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Curve>;
}

impl Clone for Box<dyn Curve> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
