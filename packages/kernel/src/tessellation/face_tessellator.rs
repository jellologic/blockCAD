use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;
use crate::topology::face::FaceId;

use super::ear_clip;
use super::mesh::TriMesh;
use super::params::TessellationParams;

/// Tessellate a single planar face into triangles.
///
/// Algorithm:
/// 1. Get face's outer loop vertices (from edges/coedges)
/// 2. Project to 2D using the face's plane
/// 3. Triangulate using ear-clipping
/// 4. Map back to 3D with normals
pub fn tessellate_face(
    brep: &BRep,
    face_id: FaceId,
    face_index: u32,
    _params: &TessellationParams,
) -> KernelResult<TriMesh> {
    let face = brep.faces.get(face_id)?;
    let loop_id = face.outer_loop.ok_or_else(|| {
        KernelError::Topology("Face has no outer loop".into())
    })?;
    let loop_ = brep.loops.get(loop_id)?;

    // Get the surface (must be a plane for now)
    let surf_idx = face.surface_index.ok_or_else(|| {
        KernelError::Topology("Face has no surface".into())
    })?;
    let surface = &brep.surfaces[surf_idx];
    let normal = surface.normal_at(0.0, 0.0)?;

    // Collect 3D vertices from the loop's coedges
    let mut vertices_3d: Vec<crate::geometry::Pt3> = Vec::new();
    for &coedge_id in &loop_.coedges {
        let coedge = brep.coedges.get(coedge_id)?;
        let edge = brep.edges.get(coedge.edge)?;
        // Get the start vertex of this coedge
        let start_vid = match coedge.orientation {
            crate::topology::edge::Orientation::Forward => edge.start,
            crate::topology::edge::Orientation::Reversed => edge.end,
        };
        let vertex = brep.vertices.get(start_vid)?;
        vertices_3d.push(vertex.point);
    }

    if vertices_3d.len() < 3 {
        return Err(KernelError::Topology("Face has fewer than 3 vertices".into()));
    }

    // Project to 2D using closest_parameters on the surface
    let vertices_2d: Vec<[f64; 2]> = vertices_3d
        .iter()
        .map(|p| {
            surface.closest_parameters(p, 1e-9).map(|(u, v)| [u, v])
        })
        .collect::<KernelResult<Vec<_>>>()?;

    // Collect inner loop vertices
    let mut inner_loops_3d: Vec<Vec<crate::geometry::Pt3>> = Vec::new();
    let mut inner_loops_2d: Vec<Vec<[f64; 2]>> = Vec::new();

    for &inner_loop_id in &face.inner_loops {
        let inner_loop = brep.loops.get(inner_loop_id)?;
        let mut inner_verts_3d: Vec<crate::geometry::Pt3> = Vec::new();
        for &coedge_id in &inner_loop.coedges {
            let coedge = brep.coedges.get(coedge_id)?;
            let edge = brep.edges.get(coedge.edge)?;
            let start_vid = match coedge.orientation {
                crate::topology::edge::Orientation::Forward => edge.start,
                crate::topology::edge::Orientation::Reversed => edge.end,
            };
            let vertex = brep.vertices.get(start_vid)?;
            inner_verts_3d.push(vertex.point);
        }
        let inner_verts_2d: Vec<[f64; 2]> = inner_verts_3d
            .iter()
            .map(|p| surface.closest_parameters(p, 1e-9).map(|(u, v)| [u, v]))
            .collect::<KernelResult<Vec<_>>>()?;
        inner_loops_3d.push(inner_verts_3d);
        inner_loops_2d.push(inner_verts_2d);
    }

    // Triangulate in 2D
    let mut triangles = if inner_loops_2d.is_empty() {
        ear_clip::triangulate(&vertices_2d)
    } else {
        ear_clip::triangulate_with_holes(&vertices_2d, &inner_loops_2d)
    };

    // Build combined vertex arrays (outer + inner loops) with pre-computed capacity
    let inner_total: usize = inner_loops_3d.iter().map(|v| v.len()).sum();
    let total_verts = vertices_3d.len() + inner_total;

    let mut all_verts_3d = Vec::with_capacity(total_verts);
    all_verts_3d.extend_from_slice(&vertices_3d);
    for inner in &inner_loops_3d {
        all_verts_3d.extend_from_slice(inner);
    }

    let mut all_verts_2d = Vec::with_capacity(total_verts);
    all_verts_2d.extend_from_slice(&vertices_2d);
    for inner in &inner_loops_2d {
        all_verts_2d.extend_from_slice(inner);
    }

    // For faces with inner loops: merge coincident vertices (same 3D position)
    // to the canonical first index, then filter degenerate triangles
    if !inner_loops_3d.is_empty() {
        let tol2: f64 = 1e-18;
        let n_verts = all_verts_3d.len();
        let mut canonical: Vec<usize> = (0..n_verts).collect();
        for i in 1..n_verts {
            for j in 0..i {
                let dx = all_verts_3d[i].x - all_verts_3d[j].x;
                let dy = all_verts_3d[i].y - all_verts_3d[j].y;
                let dz = all_verts_3d[i].z - all_verts_3d[j].z;
                if dx * dx + dy * dy + dz * dz < tol2 {
                    canonical[i] = canonical[j];
                    break;
                }
            }
        }

        // Remap triangle indices and filter degenerates
        triangles = triangles
            .into_iter()
            .map(|tri| [canonical[tri[0]], canonical[tri[1]], canonical[tri[2]]])
            .filter(|tri| tri[0] != tri[1] && tri[1] != tri[2] && tri[0] != tri[2])
            .collect();
    }

    // Build TriMesh
    let mut mesh = TriMesh::new();

    // Add all vertices
    for p in &all_verts_3d {
        mesh.positions.push(p.x as f32);
        mesh.positions.push(p.y as f32);
        mesh.positions.push(p.z as f32);
        mesh.normals.push(normal.x as f32);
        mesh.normals.push(normal.y as f32);
        mesh.normals.push(normal.z as f32);
    }
    // UVs from 2D projection
    for uv in &all_verts_2d {
        mesh.uvs.push(uv[0] as f32);
        mesh.uvs.push(uv[1] as f32);
    }

    // Add triangle indices
    for tri in &triangles {
        mesh.indices.push(tri[0] as u32);
        mesh.indices.push(tri[1] as u32);
        mesh.indices.push(tri[2] as u32);
        mesh.face_ids.push(face_index);
    }

    Ok(mesh)
}

/// Tessellate an entire BRep into a single triangle mesh.
pub fn tessellate_brep(
    brep: &BRep,
    params: &TessellationParams,
) -> KernelResult<TriMesh> {
    let mut combined = TriMesh::new();
    let mut face_index = 0u32;

    for (face_id, _face) in brep.faces.iter() {
        let face_mesh = tessellate_face(brep, face_id, face_index, params)?;
        combined.merge(&face_mesh);
        face_index += 1;
    }

    combined.fix_winding();
    combined.validate()?;
    Ok(combined)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn tessellate_box() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();
        let params = TessellationParams::default();
        let mesh = tessellate_brep(&brep, &params).unwrap();

        // 6 faces × 2 triangles each = 12 triangles
        assert_eq!(mesh.triangle_count(), 12, "Box should have 12 triangles");
        // 6 faces × 4 vertices each = 24 vertices
        assert_eq!(mesh.vertex_count(), 24);
        // Mesh should be valid
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn tessellate_unit_cube() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let params = TessellationParams::default();
        let mesh = tessellate_brep(&brep, &params).unwrap();
        assert_eq!(mesh.triangle_count(), 12);
        assert!(mesh.validate().is_ok());
    }

    #[test]
    fn tessellated_box_to_bytes() {
        let brep = build_box_brep(1.0, 1.0, 1.0).unwrap();
        let params = TessellationParams::default();
        let mesh = tessellate_brep(&brep, &params).unwrap();
        let bytes = mesh.to_bytes();
        // Should have data
        assert!(bytes.len() > 100);
        // First 4 bytes = vertex count
        let vc = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        assert_eq!(vc, 24);
    }
}
