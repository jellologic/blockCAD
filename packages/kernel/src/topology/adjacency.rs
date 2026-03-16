use crate::geometry::Pt3;
use crate::topology::face::FaceId;
use crate::topology::vertex::VertexId;
use crate::topology::BRep;
use crate::topology::edge::Orientation;

/// A shared geometric edge between two faces.
#[derive(Debug)]
pub struct SharedEdge {
    pub start: Pt3,
    pub end: Pt3,
    pub face_a: FaceId,
    pub face_b: FaceId,
    /// Vertex IDs on face A that match start/end
    pub vertex_a_start: VertexId,
    pub vertex_a_end: VertexId,
    /// Vertex IDs on face B that match start/end
    pub vertex_b_start: VertexId,
    pub vertex_b_end: VertexId,
}

/// Find all geometric edges shared between exactly two faces.
/// Since each face creates its own vertices (no sharing), we compare
/// vertex positions within tolerance.
pub fn find_shared_edges(brep: &BRep, tol: f64) -> Vec<SharedEdge> {
    struct FaceEdge {
        start: Pt3,
        end: Pt3,
        face_id: FaceId,
        start_vid: VertexId,
        end_vid: VertexId,
    }

    let mut face_edges: Vec<FaceEdge> = Vec::new();

    for (face_id, face) in brep.faces.iter() {
        let loop_id = match face.outer_loop {
            Some(id) => id,
            None => continue,
        };
        let loop_ = match brep.loops.get(loop_id) {
            Ok(l) => l,
            Err(_) => continue,
        };

        for &coedge_id in &loop_.coedges {
            let coedge = match brep.coedges.get(coedge_id) {
                Ok(ce) => ce,
                Err(_) => continue,
            };
            let edge = match brep.edges.get(coedge.edge) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let (start_vid, end_vid) = match coedge.orientation {
                Orientation::Forward => (edge.start, edge.end),
                Orientation::Reversed => (edge.end, edge.start),
            };

            let start = match brep.vertices.get(start_vid) {
                Ok(v) => v.point,
                Err(_) => continue,
            };
            let end = match brep.vertices.get(end_vid) {
                Ok(v) => v.point,
                Err(_) => continue,
            };

            face_edges.push(FaceEdge {
                start,
                end,
                face_id,
                start_vid,
                end_vid,
            });
        }
    }

    // Find pairs of edges from different faces that match geometrically
    let mut shared = Vec::new();
    let tol2 = tol * tol;

    for i in 0..face_edges.len() {
        for j in (i + 1)..face_edges.len() {
            if face_edges[i].face_id == face_edges[j].face_id {
                continue;
            }

            let a = &face_edges[i];
            let b = &face_edges[j];

            let match_same = dist2(a.start, b.start) < tol2 && dist2(a.end, b.end) < tol2;
            let match_rev = dist2(a.start, b.end) < tol2 && dist2(a.end, b.start) < tol2;

            if match_same {
                shared.push(SharedEdge {
                    start: a.start,
                    end: a.end,
                    face_a: a.face_id,
                    face_b: b.face_id,
                    vertex_a_start: a.start_vid,
                    vertex_a_end: a.end_vid,
                    vertex_b_start: b.start_vid,
                    vertex_b_end: b.end_vid,
                });
            } else if match_rev {
                shared.push(SharedEdge {
                    start: a.start,
                    end: a.end,
                    face_a: a.face_id,
                    face_b: b.face_id,
                    vertex_a_start: a.start_vid,
                    vertex_a_end: a.end_vid,
                    vertex_b_start: b.end_vid,
                    vertex_b_end: b.start_vid,
                });
            }
        }
    }

    shared
}

fn dist2(a: Pt3, b: Pt3) -> f64 {
    let d = a - b;
    d.x * d.x + d.y * d.y + d.z * d.z
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn box_has_12_shared_edges() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let edges = find_shared_edges(&brep, 1e-9);
        assert_eq!(
            edges.len(),
            12,
            "Unit cube should have 12 shared edges, got {}",
            edges.len()
        );
    }

    #[test]
    fn adjacency_finds_both_faces() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        let edges = find_shared_edges(&brep, 1e-9);
        for edge in &edges {
            assert_ne!(
                edge.face_a, edge.face_b,
                "Shared edge should connect different faces"
            );
        }
    }
}
