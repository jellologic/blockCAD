use approx::assert_relative_eq;
use blockcad_kernel::geometry::curve::line::Line3;
use blockcad_kernel::geometry::curve::arc::Arc3;
use blockcad_kernel::geometry::curve::circle::Circle3;
use blockcad_kernel::geometry::curve::nurbs::NurbsCurve;
use blockcad_kernel::geometry::curve::Curve;
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::geometry::surface::cylinder::CylindricalSurface;
use blockcad_kernel::geometry::surface::Surface;
use blockcad_kernel::geometry::{Pt3, Vec3};

#[test]
fn line_endpoints() {
    let line = Line3::new(Pt3::new(1.0, 2.0, 3.0), Pt3::new(4.0, 5.0, 6.0)).unwrap();
    let start = line.point_at(0.0).unwrap();
    let end = line.point_at(1.0).unwrap();
    assert_relative_eq!(start, Pt3::new(1.0, 2.0, 3.0), epsilon = 1e-9);
    assert_relative_eq!(end, Pt3::new(4.0, 5.0, 6.0), epsilon = 1e-9);
}

#[test]
fn line_second_derivative_is_zero() {
    let line = Line3::new(Pt3::origin(), Pt3::new(1.0, 1.0, 1.0)).unwrap();
    let d2 = line.second_derivative_at(0.5).unwrap();
    assert_relative_eq!(d2.norm(), 0.0, epsilon = 1e-9);
}

#[test]
fn arc_midpoint_on_circle() {
    let arc = Arc3::new(
        Pt3::origin(),
        2.0,
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
        0.0,
        std::f64::consts::PI,
    )
    .unwrap();
    let mid = arc.point_at(0.5).unwrap();
    // Midpoint of semicircle should be at (0, 2, 0)
    assert_relative_eq!(mid.x, 0.0, epsilon = 1e-9);
    assert_relative_eq!(mid.y, 2.0, epsilon = 1e-9);
}

#[test]
fn circle_not_equal_to_arc() {
    // Circle is always closed, arc may not be
    let circle = Circle3::new(
        Pt3::origin(),
        1.0,
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
    )
    .unwrap();
    assert!(circle.is_closed());

    let arc = Arc3::new(
        Pt3::origin(),
        1.0,
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
        0.0,
        std::f64::consts::PI,
    )
    .unwrap();
    assert!(!arc.is_closed());
}

#[test]
fn plane_roundtrip_point() {
    let plane = Plane::xy(5.0);
    let (u, v) = plane.closest_parameters(&Pt3::new(3.0, 7.0, 5.0), 1e-9).unwrap();
    let reconstructed = plane.point_at(u, v).unwrap();
    assert_relative_eq!(reconstructed.x, 3.0, epsilon = 1e-9);
    assert_relative_eq!(reconstructed.y, 7.0, epsilon = 1e-9);
    assert_relative_eq!(reconstructed.z, 5.0, epsilon = 1e-9);
}

#[test]
fn cylinder_surface_height() {
    let cyl = CylindricalSurface::new(
        Pt3::origin(),
        Vec3::new(0.0, 0.0, 1.0),
        Vec3::new(1.0, 0.0, 0.0),
        1.0,
    )
    .unwrap();
    let p = cyl.point_at(0.0, 5.0).unwrap();
    assert_relative_eq!(p.z, 5.0, epsilon = 1e-9);
    assert_relative_eq!(p.x, 1.0, epsilon = 1e-9);
}

#[test]
fn nurbs_curve_bounding_box_conservative() {
    let c = NurbsCurve::new(
        vec![
            Pt3::new(0.0, 0.0, 0.0),
            Pt3::new(0.5, 1.0, 0.0),
            Pt3::new(1.0, 0.0, 0.0),
        ],
        vec![1.0, 1.0, 1.0],
        vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        2,
    )
    .unwrap();
    let bb = c.bounding_box().unwrap();
    // BBox from control points is conservative (actual curve lies within)
    assert!(bb.contains(&Pt3::new(0.5, 0.5, 0.0)));
}
