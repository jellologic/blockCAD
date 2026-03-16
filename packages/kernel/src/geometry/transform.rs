//! 4×4 homogeneous transformation matrix utilities for assembly component positioning.

use super::{Mat4, Pt3, Vec3};

/// Create an identity transform.
pub fn identity() -> Mat4 {
    Mat4::identity()
}

/// Create a translation transform.
pub fn translation(x: f64, y: f64, z: f64) -> Mat4 {
    Mat4::new_translation(&Vec3::new(x, y, z))
}

/// Create a rotation transform from axis-angle representation.
/// `axis` is the rotation axis (will be normalized), `angle` is in radians.
pub fn rotation_axis_angle(axis: &Vec3, angle: f64) -> Mat4 {
    let len = axis.norm();
    if len < 1e-12 {
        return Mat4::identity();
    }
    let unit_axis = nalgebra::Unit::new_normalize(*axis);
    let rot = nalgebra::Rotation3::from_axis_angle(&unit_axis, angle);
    rot.to_homogeneous()
}

/// Compose two transforms: result = a * b (a applied after b).
pub fn compose(a: &Mat4, b: &Mat4) -> Mat4 {
    a * b
}

/// Transform a 3D point by a 4×4 matrix (applies translation).
pub fn transform_point(m: &Mat4, p: &Pt3) -> Pt3 {
    let h = m * nalgebra::Vector4::new(p.x, p.y, p.z, 1.0);
    Pt3::new(h.x, h.y, h.z)
}

/// Transform a 3D normal vector by a 4×4 matrix (no translation, uses inverse-transpose).
pub fn transform_normal(m: &Mat4, n: &Vec3) -> Vec3 {
    // For rigid transforms (rotation + translation), the upper-left 3×3 is orthogonal,
    // so we can just multiply by it directly.
    let r = m.fixed_view::<3, 3>(0, 0);
    let result = r * n;
    result.normalize()
}

/// Extract the translation component from a 4×4 matrix.
pub fn get_translation(m: &Mat4) -> Vec3 {
    Vec3::new(m[(0, 3)], m[(1, 3)], m[(2, 3)])
}

/// Create a transform from translation + axis-angle rotation.
pub fn from_translation_rotation(tx: f64, ty: f64, tz: f64, axis: &Vec3, angle: f64) -> Mat4 {
    compose(&translation(tx, ty, tz), &rotation_axis_angle(axis, angle))
}

/// Convert a Mat4 to a flat [f64; 16] array (column-major, matching nalgebra storage).
pub fn to_array(m: &Mat4) -> [f64; 16] {
    let mut arr = [0.0; 16];
    arr.copy_from_slice(m.as_slice());
    arr
}

/// Create a Mat4 from a flat [f64; 16] array (column-major).
pub fn from_array(arr: &[f64; 16]) -> Mat4 {
    Mat4::from_column_slice(arr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_preserves_point() {
        let m = identity();
        let p = Pt3::new(1.0, 2.0, 3.0);
        let result = transform_point(&m, &p);
        assert!((result - p).norm() < 1e-12);
    }

    #[test]
    fn translation_moves_point() {
        let m = translation(10.0, 20.0, 30.0);
        let p = Pt3::new(1.0, 2.0, 3.0);
        let result = transform_point(&m, &p);
        assert!((result.x - 11.0).abs() < 1e-12);
        assert!((result.y - 22.0).abs() < 1e-12);
        assert!((result.z - 33.0).abs() < 1e-12);
    }

    #[test]
    fn rotation_90_degrees_z() {
        let m = rotation_axis_angle(&Vec3::new(0.0, 0.0, 1.0), std::f64::consts::FRAC_PI_2);
        let p = Pt3::new(1.0, 0.0, 0.0);
        let result = transform_point(&m, &p);
        assert!((result.x).abs() < 1e-9);
        assert!((result.y - 1.0).abs() < 1e-9);
        assert!((result.z).abs() < 1e-9);
    }

    #[test]
    fn compose_translation_and_rotation() {
        let t = translation(5.0, 0.0, 0.0);
        let r = rotation_axis_angle(&Vec3::new(0.0, 0.0, 1.0), std::f64::consts::FRAC_PI_2);
        let m = compose(&t, &r); // rotate then translate
        let p = Pt3::new(1.0, 0.0, 0.0);
        let result = transform_point(&m, &p);
        // Rotate (1,0,0) by 90° Z → (0,1,0), then translate +5 X → (5,1,0)
        assert!((result.x - 5.0).abs() < 1e-9);
        assert!((result.y - 1.0).abs() < 1e-9);
    }

    #[test]
    fn transform_normal_preserves_direction() {
        let m = translation(100.0, 200.0, 300.0);
        let n = Vec3::new(0.0, 0.0, 1.0);
        let result = transform_normal(&m, &n);
        assert!((result - n).norm() < 1e-12);
    }

    #[test]
    fn array_roundtrip() {
        let m = translation(1.0, 2.0, 3.0);
        let arr = to_array(&m);
        let m2 = from_array(&arr);
        assert!((m - m2).norm() < 1e-12);
    }
}
