use crate::error::{KernelError, KernelResult};
use crate::geometry::{Pt3, Vec3};
use crate::operations::revolve::rotate_point_around_axis;
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CircularPatternParams {
    pub axis_origin: Pt3,
    pub axis_direction: Vec3,
    pub count: u32,
    /// Total angle to distribute instances over (2*PI for full circle)
    pub total_angle: f64,
}

#[derive(Debug)]
pub struct CircularPatternOp;

impl Operation for CircularPatternOp {
    type Params = CircularPatternParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        circular_pattern(input, params)
    }

    fn name(&self) -> &'static str {
        "Circular Pattern"
    }
}

pub fn circular_pattern(brep: &BRep, params: &CircularPatternParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "circular_pattern".into(),
            detail: "Cannot pattern: no existing geometry".into(),
        });
    }
    if params.count < 1 {
        return Err(KernelError::InvalidParameter {
            param: "count".into(),
            value: params.count.to_string(),
        });
    }

    let base_faces = extract_face_polygons(brep)?;
    let axis_dir = params.axis_direction.normalize();
    let angle_step = params.total_angle / params.count as f64;

    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    for i in 0..params.count {
        let angle = angle_step * i as f64;

        for (pts, normal) in &base_faces {
            let rotated_pts: Vec<Pt3> = pts.iter()
                .map(|p| rotate_point_around_axis(*p, params.axis_origin, axis_dir, angle))
                .collect();
            let rotated_normal = {
                let origin = Pt3::new(0.0, 0.0, 0.0);
                let normal_pt = Pt3::new(normal.x, normal.y, normal.z);
                let rotated = rotate_point_around_axis(normal_pt, origin, axis_dir, angle);
                Vec3::new(rotated.x, rotated.y, rotated.z)
            };
            all_faces.push((rotated_pts, rotated_normal));
        }
    }

    rebuild_brep_from_faces(&all_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    #[test]
    fn circular_pattern_creates_n_copies() {
        let brep = build_box_brep(3.0, 3.0, 3.0).unwrap();
        let params = CircularPatternParams {
            axis_origin: Pt3::new(0.0, 0.0, 0.0),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            count: 4,
            total_angle: 2.0 * std::f64::consts::PI,
        };
        let result = circular_pattern(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 24); // 6 * 4
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn circular_pattern_empty_brep_rejected() {
        let brep = BRep::new();
        let params = CircularPatternParams {
            axis_origin: Pt3::new(0.0, 0.0, 0.0),
            axis_direction: Vec3::new(0.0, 0.0, 1.0),
            count: 4,
            total_angle: 2.0 * std::f64::consts::PI,
        };
        assert!(circular_pattern(&brep, &params).is_err());
    }
}
