//! Smart mate suggestion — auto-detect appropriate mate types from face geometry.

use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;
use crate::topology::builders::extract_face_polygons;
use super::MateKind;

/// Face geometry classification for smart mate detection.
#[derive(Debug, Clone)]
enum FaceType {
    Planar { normal: Vec3 },
    NonPlanar,
    Unknown,
}

/// Classify a face from a BRep by its index (using extracted face polygons).
fn classify_face(brep: &BRep, face_idx: usize) -> FaceType {
    let polygons = match extract_face_polygons(brep) {
        Ok(p) => p,
        Err(_) => return FaceType::Unknown,
    };

    if face_idx >= polygons.len() {
        return FaceType::Unknown;
    }

    let (ref points, ref normal) = polygons[face_idx];
    if points.len() < 3 {
        return FaceType::Unknown;
    }

    // Check if face is planar: all points should be approximately coplanar
    let center = Pt3::new(
        points.iter().map(|p| p.x).sum::<f64>() / points.len() as f64,
        points.iter().map(|p| p.y).sum::<f64>() / points.len() as f64,
        points.iter().map(|p| p.z).sum::<f64>() / points.len() as f64,
    );

    let is_planar = points.iter().all(|p| {
        let d = (p - center).dot(normal).abs();
        d < 1e-4
    });

    if is_planar {
        FaceType::Planar { normal: *normal }
    } else {
        FaceType::NonPlanar
    }
}

/// Suggest a mate kind based on face geometry of two components.
///
/// Auto-detects:
/// - Two parallel planes -> Coincident
/// - Two perpendicular planes -> Perpendicular
/// - Two non-planar faces -> Concentric
/// - Plane + non-planar -> Tangent
pub fn suggest_mate(brep_a: &BRep, face_a: usize, brep_b: &BRep, face_b: usize) -> MateKind {
    let type_a = classify_face(brep_a, face_a);
    let type_b = classify_face(brep_b, face_b);

    match (&type_a, &type_b) {
        (FaceType::Planar { normal: na }, FaceType::Planar { normal: nb }) => {
            let dot = na.dot(nb).abs();
            if dot > 0.95 {
                MateKind::Coincident
            } else if dot < 0.05 {
                MateKind::Perpendicular
            } else {
                let angle = na.dot(nb).acos();
                MateKind::Angle { value: angle }
            }
        }
        (FaceType::NonPlanar, FaceType::NonPlanar) => {
            MateKind::Concentric
        }
        (FaceType::Planar { .. }, FaceType::NonPlanar)
        | (FaceType::NonPlanar, FaceType::Planar { .. }) => {
            MateKind::Tangent
        }
        _ => {
            MateKind::Coincident
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn box_faces_suggest_geometric_mate() {
        let brep = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let suggestion = suggest_mate(&brep, 0, &brep, 1);
        // Two faces of a box: should suggest coincident, perpendicular, or angle
        match suggestion {
            MateKind::Coincident | MateKind::Perpendicular | MateKind::Angle { .. } => {}
            _ => panic!("Expected geometric mate, got {:?}", suggestion),
        }
    }

    #[test]
    fn same_face_suggests_coincident() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let suggestion = suggest_mate(&brep, 0, &brep, 0);
        assert!(matches!(suggestion, MateKind::Coincident));
    }

    #[test]
    fn unknown_face_index_falls_back() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let suggestion = suggest_mate(&brep, 999, &brep, 999);
        assert!(matches!(suggestion, MateKind::Coincident));
    }

    #[test]
    fn different_breps_suggest_mate() {
        let brep_a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let brep_b = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let suggestion = suggest_mate(&brep_a, 0, &brep_b, 0);
        match suggestion {
            MateKind::Coincident | MateKind::Perpendicular | MateKind::Angle { .. } => {}
            _ => panic!("Expected geometric mate for two box faces"),
        }
    }
}
