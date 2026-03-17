//! Split body operation: divides a solid into parts using a splitting plane.

use serde::{Deserialize, Serialize};

use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;

use super::csg::{CsgPlane, CsgPolygon, brep_to_polygons, polygons_to_brep};

/// Which side(s) to keep after splitting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitKeep {
    /// Keep the part above (front of) the plane.
    Above,
    /// Keep the part below (back of) the plane.
    Below,
    /// Return both parts. For now returns the above side (multi-body not yet supported).
    Both,
}

/// Parameters for the split body operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitParams {
    pub plane_origin: Pt3,
    pub plane_normal: Vec3,
    pub keep: SplitKeep,
}

/// Split a body along a plane.
///
/// Uses the existing BSP/CSG polygon splitting machinery to clip all polygons
/// against the splitting plane, then caps the cut surface with a planar face.
pub fn split_body(brep: &BRep, params: &SplitParams) -> KernelResult<BRep> {
    // Validate normal
    let norm_len = params.plane_normal.norm();
    if norm_len < 1e-12 {
        return Err(KernelError::InvalidParameter {
            param: "plane_normal".into(),
            value: format!("{:?}", params.plane_normal),
        });
    }

    let plane = CsgPlane::from_normal_and_point(&params.plane_normal, &params.plane_origin);
    let polygons = brep_to_polygons(brep)?;

    if polygons.is_empty() {
        return Err(KernelError::Operation {
            op: "split_body".into(),
            detail: "No polygons in input body".into(),
        });
    }

    // Classify and split every polygon against the splitting plane
    let mut front_polys = Vec::new();
    let mut back_polys = Vec::new();

    for poly in &polygons {
        let mut cf = Vec::new();
        let mut cb = Vec::new();
        let mut f = Vec::new();
        let mut b = Vec::new();
        super::csg::split_polygon_pub(&plane, poly, &mut cf, &mut cb, &mut f, &mut b);

        // Coplanar faces go to the side they face
        front_polys.extend(cf);
        front_polys.extend(f);
        back_polys.extend(cb);
        back_polys.extend(b);
    }

    // Pick the side to keep
    let mut kept = match params.keep {
        SplitKeep::Above => front_polys,
        SplitKeep::Below => back_polys,
        SplitKeep::Both => front_polys, // multi-body not yet supported
    };

    if kept.is_empty() {
        return Err(KernelError::Operation {
            op: "split_body".into(),
            detail: "Split plane does not intersect the body".into(),
        });
    }

    // Build cap face: collect all intersection edges on the split plane
    // by finding boundary edges of the kept polygons that lie on the plane.
    if let Some(cap) = build_cap_polygon(&kept, &plane, &params.keep) {
        kept.push(cap);
    }

    polygons_to_brep(&kept)
}

/// Build a planar cap polygon for the cut surface.
///
/// Strategy: collect vertices from kept polygons that lie on the splitting plane,
/// then compute their convex hull projected onto the plane to form a cap face.
fn build_cap_polygon(
    kept_polys: &[CsgPolygon],
    plane: &CsgPlane,
    keep: &SplitKeep,
) -> Option<CsgPolygon> {
    let tol = 1e-4;

    // Collect all vertices that lie on the split plane
    let mut on_plane: Vec<Pt3> = Vec::new();
    for poly in kept_polys {
        for v in &poly.vertices {
            let d = plane.signed_distance(v).abs();
            if d < tol {
                // Deduplicate
                let dominated = on_plane.iter().any(|e| {
                    let dx = e.x - v.x;
                    let dy = e.y - v.y;
                    let dz = e.z - v.z;
                    dx * dx + dy * dy + dz * dz < tol * tol
                });
                if !dominated {
                    on_plane.push(*v);
                }
            }
        }
    }

    if on_plane.len() < 3 {
        return None;
    }

    // Project onto 2D plane coordinates for convex hull
    let n = plane.normal.normalize();

    // Build orthonormal basis on the plane
    let arbitrary = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let u = Vec3::new(
        n.y * arbitrary.z - n.z * arbitrary.y,
        n.z * arbitrary.x - n.x * arbitrary.z,
        n.x * arbitrary.y - n.y * arbitrary.x,
    ).normalize();
    let v = Vec3::new(
        n.y * u.z - n.z * u.y,
        n.z * u.x - n.x * u.z,
        n.x * u.y - n.y * u.x,
    );

    // Centroid as reference
    let cx: f64 = on_plane.iter().map(|p| p.x).sum::<f64>() / on_plane.len() as f64;
    let cy: f64 = on_plane.iter().map(|p| p.y).sum::<f64>() / on_plane.len() as f64;
    let cz: f64 = on_plane.iter().map(|p| p.z).sum::<f64>() / on_plane.len() as f64;

    // Project to 2D and sort by angle for convex hull
    let mut pts_2d: Vec<(f64, f64, usize)> = on_plane
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let dx = p.x - cx;
            let dy = p.y - cy;
            let dz = p.z - cz;
            let pu = dx * u.x + dy * u.y + dz * u.z;
            let pv = dx * v.x + dy * v.y + dz * v.z;
            (pu, pv, i)
        })
        .collect();

    // Convex hull using gift wrapping
    let hull_indices = convex_hull_2d(&pts_2d);
    if hull_indices.len() < 3 {
        return None;
    }

    let vertices: Vec<Pt3> = hull_indices.iter().map(|&i| on_plane[pts_2d[i].2]).collect();

    // Cap normal should point inward (opposite to the kept side's outward direction)
    let cap_normal = match keep {
        SplitKeep::Above => -n,  // Cap faces downward (back into the kept volume)
        SplitKeep::Below => n,   // Cap faces upward (back into the kept volume)
        SplitKeep::Both => -n,   // Same as Above for now
    };

    Some(CsgPolygon {
        vertices,
        normal: cap_normal,
    })
}

/// Simple 2D convex hull via gift wrapping. Returns indices into pts_2d.
fn convex_hull_2d(pts: &[(f64, f64, usize)]) -> Vec<usize> {
    let n = pts.len();
    if n < 3 {
        return (0..n).collect();
    }

    // Find leftmost point
    let mut start = 0;
    for i in 1..n {
        if pts[i].0 < pts[start].0 || (pts[i].0 == pts[start].0 && pts[i].1 < pts[start].1) {
            start = i;
        }
    }

    let mut hull = Vec::new();
    let mut current = start;
    loop {
        hull.push(current);
        let mut next = 0;
        for i in 0..n {
            if i == current {
                continue;
            }
            if next == current {
                next = i;
                continue;
            }
            let cross = cross_2d(pts[current], pts[next], pts[i]);
            if cross < 0.0 || (cross.abs() < 1e-12 && dist2_2d(pts[current], pts[i]) > dist2_2d(pts[current], pts[next])) {
                next = i;
            }
        }
        current = next;
        if current == start || hull.len() > n {
            break;
        }
    }
    hull
}

fn cross_2d(o: (f64, f64, usize), a: (f64, f64, usize), b: (f64, f64, usize)) -> f64 {
    (a.0 - o.0) * (b.1 - o.1) - (a.1 - o.1) * (b.0 - o.0)
}

fn dist2_2d(a: (f64, f64, usize), b: (f64, f64, usize)) -> f64 {
    (a.0 - b.0) * (a.0 - b.0) + (a.1 - b.1) * (a.1 - b.1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn split_box_keep_above() {
        // 10x10x10 box from (0,0,0) to (10,10,10), split at Z=5
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = SplitParams {
            plane_origin: Pt3::new(0.0, 0.0, 5.0),
            plane_normal: Vec3::new(0.0, 0.0, 1.0),
            keep: SplitKeep::Above,
        };
        let result = split_body(&brep, &params).unwrap();
        assert!(result.faces.len() >= 5, "Split above should have >= 5 faces, got {}", result.faces.len());
    }

    #[test]
    fn split_box_keep_below() {
        // 10x10x10 box from (0,0,0) to (10,10,10), split at Z=5
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = SplitParams {
            plane_origin: Pt3::new(0.0, 0.0, 5.0),
            plane_normal: Vec3::new(0.0, 0.0, 1.0),
            keep: SplitKeep::Below,
        };
        let result = split_body(&brep, &params).unwrap();
        assert!(result.faces.len() >= 5, "Split below should have >= 5 faces, got {}", result.faces.len());
    }

    #[test]
    fn split_box_at_angle() {
        // 10x10x10 box, split with diagonal plane through center
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = SplitParams {
            plane_origin: Pt3::new(5.0, 5.0, 5.0),
            plane_normal: Vec3::new(1.0, 1.0, 0.0),
            keep: SplitKeep::Above,
        };
        let result = split_body(&brep, &params).unwrap();
        assert!(result.faces.len() >= 3, "Angled split should produce faces, got {}", result.faces.len());
    }

    #[test]
    fn split_invalid_zero_normal() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let params = SplitParams {
            plane_origin: Pt3::new(5.0, 5.0, 5.0),
            plane_normal: Vec3::new(0.0, 0.0, 0.0),
            keep: SplitKeep::Above,
        };
        let result = split_body(&brep, &params);
        assert!(result.is_err(), "Zero normal should fail");
        if let Err(KernelError::InvalidParameter { param, .. }) = &result {
            assert_eq!(param, "plane_normal");
        } else {
            panic!("Expected InvalidParameter error, got {:?}", result);
        }
    }

    #[test]
    fn split_plane_outside_body() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        // Plane far above the box — all polygons below, none above
        let params = SplitParams {
            plane_origin: Pt3::new(0.0, 0.0, 100.0),
            plane_normal: Vec3::new(0.0, 0.0, 1.0),
            keep: SplitKeep::Above,
        };
        let result = split_body(&brep, &params);
        assert!(result.is_err(), "Plane outside body (above side empty) should fail");
    }
}
