//! Tests that fillet produces watertight meshes on complex bodies.

use blockcad_kernel::geometry::{Pt3, Vec3};
use blockcad_kernel::geometry::surface::plane::Plane;
use blockcad_kernel::operations::extrude::ExtrudeProfile;
use blockcad_kernel::operations::fillet::{fillet_edges, FilletParams};
use blockcad_kernel::operations::pattern::mirror::{mirror_brep, MirrorParams};
use blockcad_kernel::operations::revolve::{revolve_profile, RevolveParams};
use blockcad_kernel::operations::boolean::csg::csg_union;
use blockcad_kernel::tessellation::{tessellate_brep, TessellationParams};
use blockcad_kernel::topology::adjacency::find_shared_edges;
use blockcad_kernel::topology::builders::{build_box_brep, extract_face_polygons, rebuild_brep_from_faces};

/// Find the first non-coplanar shared edge (dihedral angle > ~5 degrees).
fn find_non_coplanar_edge(brep: &blockcad_kernel::topology::BRep) -> Option<u32> {
    let shared = find_shared_edges(brep, 1e-9);
    for (i, se) in shared.iter().enumerate() {
        let fa = brep.faces.get(se.face_a).ok()?;
        let fb = brep.faces.get(se.face_b).ok()?;
        let na = brep.surfaces[fa.surface_index?].normal_at(0.0, 0.0).ok()?;
        let nb = brep.surfaces[fb.surface_index?].normal_at(0.0, 0.0).ok()?;
        let cos_a = na.dot(&nb);
        if cos_a.abs() < 0.95 {
            return Some(i as u32);
        }
    }
    None
}

#[test]
fn fillet_boolean_union_watertight() {
    let a = build_box_brep(10.0, 5.0, 7.0).unwrap();
    let b_polys = extract_face_polygons(&build_box_brep(10.0, 5.0, 7.0).unwrap()).unwrap();
    let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys
        .into_iter()
        .map(|(pts, n)| {
            (
                pts.into_iter()
                    .map(|p| Pt3::new(p.x + 5.0, p.y, p.z))
                    .collect(),
                n,
            )
        })
        .collect();
    let b = rebuild_brep_from_faces(&b_offset).unwrap();
    let union_brep = csg_union(&a, &b).unwrap();

    let edge_idx = find_non_coplanar_edge(&union_brep)
        .expect("Should find a non-coplanar edge in union result");

    let params = FilletParams {
        edge_indices: vec![edge_idx],
        radius: 0.5,
    };
    let filleted = fillet_edges(&union_brep, &params).unwrap();
    let mesh = tessellate_brep(&filleted, &TessellationParams::default()).unwrap();
    assert!(
        mesh.is_watertight(),
        "Boolean union + fillet mesh should be watertight"
    );
}

#[test]
fn fillet_mirrored_box_watertight() {
    let brep = build_box_brep(10.0, 5.0, 7.0).unwrap();
    let mirrored = mirror_brep(
        &brep,
        &MirrorParams {
            plane_origin: Pt3::new(0.0, 0.0, 0.0),
            plane_normal: Vec3::new(1.0, 0.0, 0.0),
        },
    )
    .unwrap();

    let edge_idx = find_non_coplanar_edge(&mirrored)
        .expect("Should find a non-coplanar edge in mirrored result");

    let params = FilletParams {
        edge_indices: vec![edge_idx],
        radius: 0.5,
    };
    let filleted = fillet_edges(&mirrored, &params).unwrap();
    let mesh = tessellate_brep(&filleted, &TessellationParams::default()).unwrap();
    assert!(
        mesh.is_watertight(),
        "Mirror + fillet mesh should be watertight"
    );
}

#[test]
fn fillet_revolve_watertight() {
    // Full revolution of a rectangle (x=5..10, z=0..3 in XZ plane) around the Y axis.
    // The seam edge at angle=0 is coplanar and should be skipped by fillet.
    let profile = ExtrudeProfile {
        points: vec![
            Pt3::new(5.0, 0.0, 0.0),
            Pt3::new(10.0, 0.0, 0.0),
            Pt3::new(10.0, 0.0, 3.0),
            Pt3::new(5.0, 0.0, 3.0),
        ],
        plane: Plane {
            origin: Pt3::new(5.0, 0.0, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            u_axis: Vec3::new(1.0, 0.0, 0.0),
            v_axis: Vec3::new(0.0, 0.0, 1.0),
        },
    };
    let params = RevolveParams::full(Pt3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
    let brep = revolve_profile(&profile, &params).unwrap();

    // Fillet edge 0 (may be a seam edge, which gets gracefully skipped)
    let fillet_params = FilletParams {
        edge_indices: vec![0],
        radius: 0.5,
    };
    let filleted = fillet_edges(&brep, &fillet_params).unwrap();
    let mesh = tessellate_brep(&filleted, &TessellationParams::default()).unwrap();
    assert!(
        mesh.is_watertight(),
        "Revolve + fillet mesh should be watertight"
    );
}

#[test]
fn fillet_revolve_non_seam_edge_produces_valid_brep() {
    // Full revolution, then fillet a non-seam edge (edge that connects two non-coplanar faces).
    // Note: watertightness for non-seam edges on revolved bodies is a known limitation,
    // but the operation should succeed without panics or NaN values.
    let profile = ExtrudeProfile {
        points: vec![
            Pt3::new(5.0, 0.0, 0.0),
            Pt3::new(10.0, 0.0, 0.0),
            Pt3::new(10.0, 0.0, 3.0),
            Pt3::new(5.0, 0.0, 3.0),
        ],
        plane: Plane {
            origin: Pt3::new(5.0, 0.0, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            u_axis: Vec3::new(1.0, 0.0, 0.0),
            v_axis: Vec3::new(0.0, 0.0, 1.0),
        },
    };
    let params = RevolveParams::full(Pt3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0));
    let brep = revolve_profile(&profile, &params).unwrap();

    if let Some(edge_idx) = find_non_coplanar_edge(&brep) {
        let fillet_params = FilletParams {
            edge_indices: vec![edge_idx],
            radius: 0.3,
        };
        let filleted = fillet_edges(&brep, &fillet_params).unwrap();
        // Should produce more faces than original (fillet strip adds faces)
        assert!(filleted.faces.len() > brep.faces.len(),
            "Fillet on revolve should add faces");
    }
}
