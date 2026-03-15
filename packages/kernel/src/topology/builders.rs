//! Utility functions for constructing B-Rep topology from geometric data.

use crate::error::KernelResult;
use crate::geometry::curve::line::Line3;
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};

use super::body::Body;
use super::coedge::CoEdge;
use super::edge::{Edge, Orientation};
use super::face::Face;
use super::loop_::Loop;
use super::shell::Shell;
use super::solid::Solid;
use super::vertex::Vertex;
use super::brep::BRep;

/// Build a closed face from an ordered list of 3D points lying on a plane.
/// Creates vertices, edges (Line3), coedges, a loop, and a face.
/// Returns the FaceId.
pub fn make_planar_face(brep: &mut BRep, points: &[Pt3], plane: Plane) -> KernelResult<super::face::FaceId> {
    let n = points.len();
    assert!(n >= 3, "Need at least 3 points for a face");

    // Add surface
    let surf_idx = brep.add_surface(Box::new(plane));

    // Create vertices
    let vert_ids: Vec<_> = points
        .iter()
        .map(|p| brep.vertices.insert(Vertex::new(*p)))
        .collect();

    // Create edges and curves
    let mut edge_ids = Vec::with_capacity(n);
    for i in 0..n {
        let j = (i + 1) % n;
        let line = Line3::new(points[i], points[j])?;
        let curve_idx = brep.add_curve(Box::new(line));
        let edge = Edge::new(vert_ids[i], vert_ids[j]).with_curve(curve_idx);
        edge_ids.push(brep.edges.insert(edge));
    }

    // Create coedges (forward orientation for outer loop)
    let mut coedge_ids = Vec::with_capacity(n);
    for &edge_id in &edge_ids {
        let coedge = CoEdge::new(edge_id, Orientation::Forward);
        coedge_ids.push(brep.coedges.insert(coedge));
    }

    // Link coedges: next/prev
    for i in 0..n {
        let next = coedge_ids[(i + 1) % n];
        let prev = coedge_ids[(i + n - 1) % n];
        if let Ok(ce) = brep.coedges.get_mut(coedge_ids[i]) {
            ce.next = Some(next);
            ce.prev = Some(prev);
        }
    }

    // Create loop
    let loop_id = brep.loops.insert(Loop::new(coedge_ids));

    // Create face
    let face = Face::new().with_surface(surf_idx).with_outer_loop(loop_id);
    Ok(brep.faces.insert(face))
}

/// Build a box (rectangular prism) BRep from width, height, depth on the XY plane.
/// Origin is at (0, 0, 0), extends to (width, height, depth).
pub fn build_box_brep(width: f64, height: f64, depth: f64) -> KernelResult<BRep> {
    let mut brep = BRep::new();

    // 8 corner points
    let p = [
        Pt3::new(0.0, 0.0, 0.0),       // 0: bottom-front-left
        Pt3::new(width, 0.0, 0.0),      // 1: bottom-front-right
        Pt3::new(width, height, 0.0),    // 2: bottom-back-right
        Pt3::new(0.0, height, 0.0),      // 3: bottom-back-left
        Pt3::new(0.0, 0.0, depth),       // 4: top-front-left
        Pt3::new(width, 0.0, depth),     // 5: top-front-right
        Pt3::new(width, height, depth),  // 6: top-back-right
        Pt3::new(0.0, height, depth),    // 7: top-back-left
    ];

    // 6 faces with outward-pointing normals
    // Bottom face (Z=0, normal -Z)
    let bottom_plane = Plane {
        origin: p[0],
        normal: Vec3::new(0.0, 0.0, -1.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 1.0, 0.0),
    };
    make_planar_face(&mut brep, &[p[0], p[3], p[2], p[1]], bottom_plane)?;

    // Top face (Z=depth, normal +Z)
    let top_plane = Plane {
        origin: p[4],
        normal: Vec3::new(0.0, 0.0, 1.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 1.0, 0.0),
    };
    make_planar_face(&mut brep, &[p[4], p[5], p[6], p[7]], top_plane)?;

    // Front face (Y=0, normal -Y)
    let front_plane = Plane {
        origin: p[0],
        normal: Vec3::new(0.0, -1.0, 0.0),
        u_axis: Vec3::new(1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    };
    make_planar_face(&mut brep, &[p[0], p[1], p[5], p[4]], front_plane)?;

    // Back face (Y=height, normal +Y)
    let back_plane = Plane {
        origin: p[3],
        normal: Vec3::new(0.0, 1.0, 0.0),
        u_axis: Vec3::new(-1.0, 0.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    };
    make_planar_face(&mut brep, &[p[2], p[3], p[7], p[6]], back_plane)?;

    // Left face (X=0, normal -X)
    let left_plane = Plane {
        origin: p[0],
        normal: Vec3::new(-1.0, 0.0, 0.0),
        u_axis: Vec3::new(0.0, 1.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    };
    make_planar_face(&mut brep, &[p[3], p[0], p[4], p[7]], left_plane)?;

    // Right face (X=width, normal +X)
    let right_plane = Plane {
        origin: p[1],
        normal: Vec3::new(1.0, 0.0, 0.0),
        u_axis: Vec3::new(0.0, -1.0, 0.0),
        v_axis: Vec3::new(0.0, 0.0, 1.0),
    };
    make_planar_face(&mut brep, &[p[1], p[2], p[6], p[5]], right_plane)?;

    // Collect face IDs for shell
    let face_ids: Vec<_> = brep.faces.iter().map(|(id, _)| id).collect();
    let shell_id = brep.shells.insert(Shell::new(face_ids, true));
    let solid_id = brep.solids.insert(Solid::new(vec![shell_id]));
    brep.body = Body::Solid(solid_id);

    Ok(brep)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_box_topology() {
        let brep = build_box_brep(10.0, 5.0, 3.0).unwrap();
        // Box: 8 vertices per face * 6 faces = 48 total (not shared — each face owns its own)
        // Actually with make_planar_face, each face creates its own vertices
        assert_eq!(brep.faces.len(), 6);
        assert_eq!(brep.shells.len(), 1);
        assert_eq!(brep.solids.len(), 1);
    }

    #[test]
    fn box_has_six_faces() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        assert_eq!(brep.faces.len(), 6);
    }

    #[test]
    fn box_faces_have_loops() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        for (_id, face) in brep.faces.iter() {
            assert!(face.outer_loop.is_some(), "Face missing outer loop");
            assert!(face.surface_index.is_some(), "Face missing surface");
        }
    }

    #[test]
    fn box_loops_have_four_coedges() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        for (_id, face) in brep.faces.iter() {
            let loop_id = face.outer_loop.unwrap();
            let loop_ = brep.loops.get(loop_id).unwrap();
            assert_eq!(loop_.len(), 4, "Rectangular face should have 4 coedges");
        }
    }
}
