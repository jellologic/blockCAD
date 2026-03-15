pub mod bbox;
pub mod curve;
pub mod serde_helpers;
pub mod surface;

// Re-export nalgebra types as our canonical geometry primitives
pub use nalgebra::{
    Isometry3, Matrix3, Matrix4, Point2, Point3, Translation3, UnitQuaternion, Vector2, Vector3,
    Vector4,
};

/// Type aliases for readability throughout the kernel
pub type Vec2 = Vector2<f64>;
pub type Vec3 = Vector3<f64>;
pub type Vec4 = Vector4<f64>;
pub type Mat3 = Matrix3<f64>;
pub type Mat4 = Matrix4<f64>;
pub type Pt2 = Point2<f64>;
pub type Pt3 = Point3<f64>;
pub type Transform3 = Isometry3<f64>;

/// Tolerance constants used throughout the kernel
pub const GEOMETRIC_TOLERANCE: f64 = 1e-9;
pub const ANGLE_TOLERANCE: f64 = 1e-12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn type_aliases_work() {
        let p = Pt3::new(1.0, 2.0, 3.0);
        let v = Vec3::new(4.0, 5.0, 6.0);
        let result = p + v;
        assert_eq!(result, Pt3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn transform_identity() {
        let t = Transform3::identity();
        let p = Pt3::new(1.0, 2.0, 3.0);
        let result = t * p;
        assert_eq!(result, p);
    }
}
