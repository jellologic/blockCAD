use crate::error::{KernelError, KernelResult};
use crate::geometry::Vec3;
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};
use crate::topology::BRep;

use crate::geometry::Pt3;
use crate::operations::traits::Operation;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LinearPatternParams {
    pub direction: Vec3,
    pub spacing: f64,
    pub count: u32,
    /// Optional second direction for 2D patterns
    pub direction2: Option<Vec3>,
    pub spacing2: Option<f64>,
    pub count2: Option<u32>,
}

#[derive(Debug)]
pub struct LinearPatternOp;

impl Operation for LinearPatternOp {
    type Params = LinearPatternParams;

    fn execute(&self, params: &Self::Params, input: &BRep) -> KernelResult<BRep> {
        linear_pattern(input, params)
    }

    fn name(&self) -> &'static str {
        "Linear Pattern"
    }
}

pub fn linear_pattern(brep: &BRep, params: &LinearPatternParams) -> KernelResult<BRep> {
    if brep.faces.is_empty() {
        return Err(KernelError::Operation {
            op: "linear_pattern".into(),
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
    let dir = params.direction.normalize();

    let mut all_faces: Vec<(Vec<Pt3>, Vec3)> = Vec::new();

    let count1 = params.count as usize;
    let count2 = params.count2.unwrap_or(1) as usize;
    let dir2 = params.direction2.map(|d| d.normalize());
    let spacing2 = params.spacing2.unwrap_or(0.0);

    for j in 0..count2 {
        for i in 0..count1 {
            let offset1 = dir * (params.spacing * i as f64);
            let offset2 = if let Some(d2) = dir2 {
                d2 * (spacing2 * j as f64)
            } else {
                Vec3::new(0.0, 0.0, 0.0)
            };
            let total_offset = offset1 + offset2;

            for (pts, normal) in &base_faces {
                let translated: Vec<Pt3> = pts.iter().map(|p| p + total_offset).collect();
                all_faces.push((translated, *normal));
            }
        }
    }

    rebuild_brep_from_faces(&all_faces)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Vec3;
    use crate::topology::builders::build_box_brep;
    use crate::topology::body::Body;

    #[test]
    fn linear_pattern_doubles_faces() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 10.0,
            count: 2,
            direction2: None,
            spacing2: None,
            count2: None,
        };
        let result = linear_pattern(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 12); // 6 * 2
        assert!(matches!(result.body, Body::Solid(_)));
    }

    #[test]
    fn linear_pattern_count_1_unchanged() {
        let brep = build_box_brep(5.0, 5.0, 5.0).unwrap();
        let params = LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 10.0,
            count: 1,
            direction2: None,
            spacing2: None,
            count2: None,
        };
        let result = linear_pattern(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 6); // unchanged
    }

    #[test]
    fn linear_pattern_2d_grid() {
        let brep = build_box_brep(3.0, 3.0, 3.0).unwrap();
        let params = LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 5.0,
            count: 2,
            direction2: Some(Vec3::new(0.0, 1.0, 0.0)),
            spacing2: Some(5.0),
            count2: Some(3),
        };
        let result = linear_pattern(&brep, &params).unwrap();
        assert_eq!(result.faces.len(), 36); // 6 * 2 * 3
    }

    #[test]
    fn linear_pattern_empty_brep_rejected() {
        let brep = BRep::new();
        let params = LinearPatternParams {
            direction: Vec3::new(1.0, 0.0, 0.0),
            spacing: 10.0,
            count: 2,
            direction2: None,
            spacing2: None,
            count2: None,
        };
        assert!(linear_pattern(&brep, &params).is_err());
    }
}
