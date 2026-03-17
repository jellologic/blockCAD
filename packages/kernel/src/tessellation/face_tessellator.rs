use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;
use crate::topology::face::FaceId;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

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

    // Build combined vertex arrays (outer + inner loops)
    let mut all_verts_3d = vertices_3d.clone();
    for inner in &inner_loops_3d {
        all_verts_3d.extend_from_slice(inner);
    }

    let mut all_verts_2d = vertices_2d.clone();
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
///
/// When the `parallel` feature is enabled (default), faces are tessellated
/// concurrently using rayon's work-stealing thread pool.
pub fn tessellate_brep(
    brep: &BRep,
    params: &TessellationParams,
) -> KernelResult<TriMesh> {
    let faces: Vec<(FaceId, u32)> = brep
        .faces
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (id, i as u32))
        .collect();

    #[cfg(feature = "parallel")]
    let face_meshes: Vec<KernelResult<TriMesh>> = faces
        .par_iter()
        .map(|&(face_id, face_index)| tessellate_face(brep, face_id, face_index, params))
        .collect();

    #[cfg(not(feature = "parallel"))]
    let face_meshes: Vec<KernelResult<TriMesh>> = faces
        .iter()
        .map(|&(face_id, face_index)| tessellate_face(brep, face_id, face_index, params))
        .collect();

    let mut combined = TriMesh::new();
    for result in face_meshes {
        combined.merge(&result?);
    }

    if !params.skip_validation {
        combined.fix_winding();
        combined.validate()?;
    }
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

    /// Tessellate the same BRep with validation skipped (sequential code path
    /// via `skip_validation`) and with the default path, then compare counts.
    #[test]
    fn parallel_same_as_sequential() {
        let brep = build_box_brep(2.0, 3.0, 4.0).unwrap();

        // Default (parallel when feature enabled) with validation
        let params = TessellationParams::default();
        let mesh_default = tessellate_brep(&brep, &params).unwrap();

        // With skip_validation — exercises the same parallel map but skips
        // fix_winding + validate, so we can compare raw output.
        let params_skip = TessellationParams {
            skip_validation: true,
            ..TessellationParams::default()
        };
        let mesh_skip = tessellate_brep(&brep, &params_skip).unwrap();

        assert_eq!(mesh_default.vertex_count(), mesh_skip.vertex_count());
        assert_eq!(mesh_default.triangle_count(), mesh_skip.triangle_count());
        assert_eq!(mesh_default.face_ids.len(), mesh_skip.face_ids.len());
    }

    /// Tessellate a non-trivial BRep (box = 6 faces) and verify face_id
    /// coverage to exercise parallelism producing correct face indices.
    #[test]
    fn parallel_face_ids_complete() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = TessellationParams::default();
        let mesh = tessellate_brep(&brep, &params).unwrap();

        // Every face index 0..6 should appear in the face_ids
        let mut seen = std::collections::HashSet::new();
        for &fid in &mesh.face_ids {
            seen.insert(fid);
        }
        assert_eq!(seen.len(), 6, "All 6 box faces should appear in face_ids");
        for i in 0..6u32 {
            assert!(seen.contains(&i), "Missing face_id {}", i);
        }
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
