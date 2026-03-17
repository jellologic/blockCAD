//! Section view — clip meshes by a plane to visualize internal assembly structure.

use crate::tessellation::mesh::TriMesh;

/// A section cutting plane defined by normal direction and offset from origin.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SectionPlane {
    /// Unit normal of the cutting plane.
    pub normal: [f64; 3],
    /// Signed distance from origin along the normal.
    pub offset: f64,
}

impl SectionPlane {
    pub fn new(normal: [f64; 3], offset: f64) -> Self {
        Self { normal, offset }
    }

    /// Evaluate signed distance from point to plane. Positive = front side (kept).
    fn signed_distance(&self, x: f64, y: f64, z: f64) -> f64 {
        self.normal[0] * x + self.normal[1] * y + self.normal[2] * z - self.offset
    }
}

/// Clip a triangle mesh by a plane, keeping geometry on the positive side.
///
/// Triangles fully in front are kept. Triangles fully behind are discarded.
/// Triangles crossing the plane are clipped, producing 1-2 new triangles.
pub fn clip_mesh_by_plane(mesh: &TriMesh, plane: &SectionPlane) -> TriMesh {
    let mut out = TriMesh::new();

    let vc = mesh.vertex_count();
    if vc == 0 {
        return out;
    }

    // Compute signed distances for all vertices
    let dists: Vec<f64> = (0..vc)
        .map(|i| {
            let x = mesh.positions[i * 3] as f64;
            let y = mesh.positions[i * 3 + 1] as f64;
            let z = mesh.positions[i * 3 + 2] as f64;
            plane.signed_distance(x, y, z)
        })
        .collect();

    for tri in mesh.indices.chunks(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;

        let d0 = dists[i0];
        let d1 = dists[i1];
        let d2 = dists[i2];

        let front0 = d0 >= 0.0;
        let front1 = d1 >= 0.0;
        let front2 = d2 >= 0.0;

        let front_count = front0 as u8 + front1 as u8 + front2 as u8;

        if front_count == 3 {
            // All vertices in front — keep entire triangle
            add_triangle_from_source(&mut out, mesh, i0, i1, i2);
        } else if front_count == 0 {
            // All behind — discard
            continue;
        } else {
            // Crossing — clip. For simplicity, we interpolate to produce new vertices.
            clip_triangle(&mut out, mesh, &dists, i0, i1, i2, front0, front1, front2);
        }
    }

    out
}

fn add_vertex_from_source(out: &mut TriMesh, mesh: &TriMesh, idx: usize) -> u32 {
    let vi = out.vertex_count() as u32;
    out.positions.push(mesh.positions[idx * 3]);
    out.positions.push(mesh.positions[idx * 3 + 1]);
    out.positions.push(mesh.positions[idx * 3 + 2]);
    out.normals.push(mesh.normals[idx * 3]);
    out.normals.push(mesh.normals[idx * 3 + 1]);
    out.normals.push(mesh.normals[idx * 3 + 2]);
    vi
}

fn add_interpolated_vertex(out: &mut TriMesh, mesh: &TriMesh, a: usize, b: usize, t: f64) -> u32 {
    let vi = out.vertex_count() as u32;
    let t = t as f32;
    let inv = 1.0 - t;
    for k in 0..3 {
        out.positions.push(mesh.positions[a * 3 + k] * inv + mesh.positions[b * 3 + k] * t);
        out.normals.push(mesh.normals[a * 3 + k] * inv + mesh.normals[b * 3 + k] * t);
    }
    vi
}

fn add_triangle_from_source(out: &mut TriMesh, mesh: &TriMesh, i0: usize, i1: usize, i2: usize) {
    let a = add_vertex_from_source(out, mesh, i0);
    let b = add_vertex_from_source(out, mesh, i1);
    let c = add_vertex_from_source(out, mesh, i2);
    out.indices.push(a);
    out.indices.push(b);
    out.indices.push(c);
}

fn clip_triangle(
    out: &mut TriMesh,
    mesh: &TriMesh,
    dists: &[f64],
    i0: usize, i1: usize, i2: usize,
    f0: bool, f1: bool, f2: bool,
) {
    // Rearrange so that the "odd one out" is first
    let (a, b, c, da, db, dc, fa) = if f0 == f1 {
        // c is the odd one
        (i2, i0, i1, dists[i2], dists[i0], dists[i1], f2)
    } else if f1 == f2 {
        // a is the odd one
        (i0, i1, i2, dists[i0], dists[i1], dists[i2], f0)
    } else {
        // b is the odd one
        (i1, i2, i0, dists[i1], dists[i2], dists[i0], f1)
    };

    // Interpolation parameters for edge a-b and a-c
    let t_ab = da / (da - db);
    let t_ac = da / (da - dc);

    if fa {
        // a is alone in front: one triangle a, ab_mid, ac_mid
        let va = add_vertex_from_source(out, mesh, a);
        let vab = add_interpolated_vertex(out, mesh, a, b, t_ab);
        let vac = add_interpolated_vertex(out, mesh, a, c, t_ac);
        out.indices.push(va);
        out.indices.push(vab);
        out.indices.push(vac);
    } else {
        // a is alone behind: two triangles from b, c, and the two intersection points
        let vb = add_vertex_from_source(out, mesh, b);
        let vc = add_vertex_from_source(out, mesh, c);
        let vab = add_interpolated_vertex(out, mesh, a, b, t_ab);
        let vac = add_interpolated_vertex(out, mesh, a, c, t_ac);
        // Triangle 1: ab_mid, b, c
        out.indices.push(vab);
        out.indices.push(vb);
        out.indices.push(vc);
        // Triangle 2: ab_mid, c, ac_mid
        out.indices.push(vab);
        out.indices.push(vc);
        out.indices.push(vac);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_quad_mesh() -> TriMesh {
        // Two triangles forming a square in the XY plane at z=0
        // (0,0,0) (10,0,0) (10,10,0) (0,10,0)
        TriMesh {
            positions: vec![
                0.0, 0.0, 0.0,
                10.0, 0.0, 0.0,
                10.0, 10.0, 0.0,
                0.0, 10.0, 0.0,
            ],
            normals: vec![
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
                0.0, 0.0, 1.0,
            ],
            uvs: vec![],
            indices: vec![0, 1, 2, 0, 2, 3],
            face_ids: vec![],
            colors: vec![],
        }
    }

    #[test]
    fn clip_keeps_all_when_fully_in_front() {
        let mesh = make_quad_mesh();
        // Plane at x = -5, normal = (1,0,0): everything is in front
        let plane = SectionPlane::new([1.0, 0.0, 0.0], -5.0);
        let clipped = clip_mesh_by_plane(&mesh, &plane);
        assert_eq!(clipped.triangle_count(), 2);
    }

    #[test]
    fn clip_removes_all_when_fully_behind() {
        let mesh = make_quad_mesh();
        // Plane at x = 20, normal = (1,0,0): everything is behind
        let plane = SectionPlane::new([1.0, 0.0, 0.0], 20.0);
        let clipped = clip_mesh_by_plane(&mesh, &plane);
        assert_eq!(clipped.triangle_count(), 0);
    }

    #[test]
    fn clip_splits_triangles_at_midpoint() {
        let mesh = make_quad_mesh();
        // Plane at x = 5, normal = (1,0,0): cuts through the middle
        let plane = SectionPlane::new([1.0, 0.0, 0.0], 5.0);
        let clipped = clip_mesh_by_plane(&mesh, &plane);
        // Should have some triangles (exact count depends on clipping)
        assert!(clipped.triangle_count() > 0);
        assert!(clipped.triangle_count() <= 4); // at most 2 cuts × 2 tris each

        // All output vertices should have x >= 5.0 (approximately)
        for i in 0..clipped.vertex_count() {
            assert!(clipped.positions[i * 3] >= 4.99,
                "Vertex x={} should be >= 5.0", clipped.positions[i * 3]);
        }
    }
}
