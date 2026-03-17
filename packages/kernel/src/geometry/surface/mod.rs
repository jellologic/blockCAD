pub mod cylinder;
pub mod nurbs;
pub mod plane;

use crate::error::KernelResult;
use crate::geometry::{bbox::BoundingBox3, Pt3, Vec3};

/// Trait for 3D parametric surfaces.
///
/// All surfaces are parameterized over a domain (u, v).
/// Implementations must be thread-safe for parallel tessellation.
pub trait Surface: Send + Sync + std::fmt::Debug {
    /// Parameter domain (u_min, u_max, v_min, v_max)
    fn domain(&self) -> (f64, f64, f64, f64);

    /// Evaluate point at (u, v)
    fn point_at(&self, u: f64, v: f64) -> KernelResult<Pt3>;

    /// Outward-pointing surface normal at (u, v)
    fn normal_at(&self, u: f64, v: f64) -> KernelResult<Vec3>;

    /// Partial derivatives at (u, v): (dS/du, dS/dv)
    fn derivatives_at(&self, u: f64, v: f64) -> KernelResult<(Vec3, Vec3)>;

    /// Find closest (u, v) parameters to a given point
    fn closest_parameters(&self, point: &Pt3, tolerance: f64) -> KernelResult<(f64, f64)>;

    /// Axis-aligned bounding box
    fn bounding_box(&self) -> KernelResult<BoundingBox3>;

    /// Whether the surface is closed in u direction
    fn is_closed_u(&self) -> bool;

    /// Whether the surface is closed in v direction
    fn is_closed_v(&self) -> bool;

    /// Clone this surface into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Surface>;
}

impl Clone for Box<dyn Surface> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
