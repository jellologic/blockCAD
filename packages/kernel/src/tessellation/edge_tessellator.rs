use crate::error::KernelResult;
use crate::geometry::curve::Curve;
use crate::geometry::Pt3;

use super::params::TessellationParams;

/// Tessellate a curve into a polyline using adaptive subdivision.
///
/// Recursively subdivides curve segments where the midpoint deviation
/// from the chord exceeds the chord tolerance.
pub fn tessellate_curve(
    curve: &dyn Curve,
    params: &TessellationParams,
) -> KernelResult<Vec<Pt3>> {
    let (t_min, t_max) = curve.domain();

    if curve.is_closed() {
        // For closed curves, pre-split into N initial segments to bootstrap subdivision
        // (otherwise start==end gives a zero-length chord and no subdivision occurs)
        let n_initial = 4;
        let mut points = Vec::new();
        for i in 0..n_initial {
            let ta = t_min + (t_max - t_min) * (i as f64 / n_initial as f64);
            let tb = t_min + (t_max - t_min) * ((i + 1) as f64 / n_initial as f64);
            let pa = curve.point_at(ta)?;
            let pb = curve.point_at(tb)?;
            points.push(pa);
            subdivide(curve, ta, tb, &pa, &pb, params, &mut points)?;
        }
        Ok(points)
    } else {
        let start = curve.point_at(t_min)?;
        let end = curve.point_at(t_max)?;
        let mut points = vec![start];
        subdivide(curve, t_min, t_max, &start, &end, params, &mut points)?;
        points.push(end);
        Ok(points)
    }
}

fn subdivide(
    curve: &dyn Curve,
    t0: f64,
    t1: f64,
    p0: &Pt3,
    p1: &Pt3,
    params: &TessellationParams,
    points: &mut Vec<Pt3>,
) -> KernelResult<()> {
    let chord_len = (p1 - p0).norm();

    // Don't subdivide below minimum edge length
    if chord_len < params.min_edge_length {
        return Ok(());
    }

    let t_mid = (t0 + t1) * 0.5;
    let p_mid_curve = curve.point_at(t_mid)?;
    let p_mid_chord = nalgebra::center(p0, p1);

    let deviation = (p_mid_curve - p_mid_chord).norm();

    if deviation > params.chord_tolerance {
        // Subdivide left half
        subdivide(curve, t0, t_mid, p0, &p_mid_curve, params, points)?;
        points.push(p_mid_curve);
        // Subdivide right half
        subdivide(curve, t_mid, t1, &p_mid_curve, p1, params, points)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::curve::circle::Circle3;
    use crate::geometry::curve::line::Line3;
    use crate::geometry::{Pt3, Vec3};

    #[test]
    fn line_returns_two_points() {
        let line = Line3::new(Pt3::origin(), Pt3::new(10.0, 0.0, 0.0)).unwrap();
        let params = TessellationParams::default();
        let points = tessellate_curve(&line, &params).unwrap();
        // A line has zero deviation at midpoint → no subdivision
        assert_eq!(points.len(), 2);
    }

    #[test]
    fn circle_subdivides() {
        let circle = Circle3::new(
            Pt3::origin(),
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();
        let params = TessellationParams::default();
        let points = tessellate_curve(&circle, &params).unwrap();
        // Circle should have many points
        assert!(points.len() > 10, "Got only {} points", points.len());
    }

    #[test]
    fn tighter_tolerance_more_points() {
        let circle = Circle3::new(
            Pt3::origin(),
            1.0,
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .unwrap();

        let coarse = tessellate_curve(&circle, &TessellationParams::preview()).unwrap();
        let fine = tessellate_curve(&circle, &TessellationParams::high_quality()).unwrap();

        assert!(
            fine.len() > coarse.len(),
            "Fine ({}) should have more points than coarse ({})",
            fine.len(),
            coarse.len()
        );
    }
}
