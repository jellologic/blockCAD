use std::f64::consts::PI;

use crate::error::{KernelError, KernelResult};
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt3, Vec3};
use crate::operations::cut_extrude::cut_extrude;
use crate::operations::extrude::{ExtrudeParams, ExtrudeProfile};

/// Type of hole to create.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HoleType {
    Simple,
    Counterbore {
        cbore_diameter: f64,
        cbore_depth: f64,
    },
    Countersink {
        csink_diameter: f64,
        csink_angle: f64,
    },
}

/// Parameters for the Hole Wizard operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HoleParams {
    pub hole_type: HoleType,
    /// Diameter of the main hole.
    pub diameter: f64,
    /// Depth of the main hole (ignored when through_all is true).
    pub depth: f64,
    /// Center position of the hole on the target face.
    pub position: Pt3,
    /// Direction the hole goes into the material (should point inward).
    pub direction: Vec3,
    /// If true, the hole cuts all the way through the body.
    #[serde(default)]
    pub through_all: bool,
}

/// Number of segments used to approximate a circular profile.
const CIRCLE_SEGMENTS: usize = 32;

/// Generate a polygon approximating a circle in 3D.
///
/// Points are ordered counter-clockwise when viewed from the direction
/// the `normal` points toward.
fn circle_profile(center: Pt3, normal: Vec3, radius: f64, segments: usize) -> Vec<Pt3> {
    let n = normal.normalize();
    // Build a local coordinate frame on the plane perpendicular to `normal`.
    let arbitrary = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let u = n.cross(&arbitrary).normalize();
    let v = n.cross(&u).normalize();

    (0..segments)
        .map(|i| {
            let theta = 2.0 * PI * (i as f64) / (segments as f64);
            let (sin_t, cos_t) = theta.sin_cos();
            center + u * (radius * cos_t) + v * (radius * sin_t)
        })
        .collect()
}

/// Build a `Plane` for a circular profile at the given center with the given normal.
fn plane_for_circle(center: Pt3, normal: Vec3) -> Plane {
    let n = normal.normalize();
    let arbitrary = if n.x.abs() < 0.9 {
        Vec3::new(1.0, 0.0, 0.0)
    } else {
        Vec3::new(0.0, 1.0, 0.0)
    };
    let u = n.cross(&arbitrary).normalize();
    let v = n.cross(&u).normalize();
    Plane {
        origin: center,
        normal: n,
        u_axis: u,
        v_axis: v,
    }
}

/// Cut a single cylindrical hole from a body using `cut_extrude`.
fn cut_cylinder(
    stock: crate::topology::BRep,
    center: Pt3,
    direction: Vec3,
    radius: f64,
    depth: f64,
    through_all: bool,
) -> KernelResult<crate::topology::BRep> {
    let profile_pts = circle_profile(center, direction, radius, CIRCLE_SEGMENTS);
    let plane = plane_for_circle(center, direction);
    let profile = ExtrudeProfile {
        points: profile_pts,
        plane,
    };

    let mut params = ExtrudeParams::blind(direction, depth);
    if through_all {
        params.end_condition = crate::operations::extrude::EndCondition::ThroughAll;
    }

    cut_extrude(stock, &profile, &params)
}

/// Execute the Hole Wizard operation.
///
/// Creates standard holes (simple, counterbore, countersink) in existing
/// geometry by performing one or more cylindrical cuts via `cut_extrude`.
pub fn hole_wizard(
    stock: crate::topology::BRep,
    params: &HoleParams,
) -> KernelResult<crate::topology::BRep> {
    // --- Validation ---
    if params.diameter <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "diameter".into(),
            value: params.diameter.to_string(),
        });
    }
    if !params.through_all && params.depth <= 0.0 {
        return Err(KernelError::InvalidParameter {
            param: "depth".into(),
            value: params.depth.to_string(),
        });
    }
    if params.direction.norm() < 1e-12 {
        return Err(KernelError::InvalidParameter {
            param: "direction".into(),
            value: "zero-length direction vector".into(),
        });
    }

    match &params.hole_type {
        HoleType::Simple => {
            cut_cylinder(
                stock,
                params.position,
                params.direction,
                params.diameter / 2.0,
                params.depth,
                params.through_all,
            )
        }

        HoleType::Counterbore {
            cbore_diameter,
            cbore_depth,
        } => {
            if *cbore_diameter <= params.diameter {
                return Err(KernelError::InvalidParameter {
                    param: "cbore_diameter".into(),
                    value: format!(
                        "Counterbore diameter ({}) must be greater than hole diameter ({})",
                        cbore_diameter, params.diameter
                    ),
                });
            }
            if *cbore_depth <= 0.0 {
                return Err(KernelError::InvalidParameter {
                    param: "cbore_depth".into(),
                    value: cbore_depth.to_string(),
                });
            }

            // Step 1: Cut the wide counterbore pocket.
            let result = cut_cylinder(
                stock,
                params.position,
                params.direction,
                cbore_diameter / 2.0,
                *cbore_depth,
                false,
            )?;

            // Step 2: Cut the narrower through-hole starting from the
            // bottom of the counterbore.
            let dir_norm = params.direction.normalize();
            let deep_center = params.position + dir_norm * (*cbore_depth);
            let remaining_depth = if params.through_all {
                params.depth // will be overridden by ThroughAll
            } else {
                params.depth - cbore_depth
            };

            if !params.through_all && remaining_depth <= 0.0 {
                // Counterbore already deeper than hole depth — just return
                // the counterbore.
                return Ok(result);
            }

            cut_cylinder(
                result,
                deep_center,
                params.direction,
                params.diameter / 2.0,
                remaining_depth.max(0.001),
                params.through_all,
            )
        }

        HoleType::Countersink {
            csink_diameter,
            csink_angle,
        } => {
            if *csink_diameter <= params.diameter {
                return Err(KernelError::InvalidParameter {
                    param: "csink_diameter".into(),
                    value: format!(
                        "Countersink diameter ({}) must be greater than hole diameter ({})",
                        csink_diameter, params.diameter
                    ),
                });
            }
            if *csink_angle <= 0.0 || *csink_angle >= 180.0 {
                return Err(KernelError::InvalidParameter {
                    param: "csink_angle".into(),
                    value: format!(
                        "Countersink angle must be between 0 and 180 degrees, got {}",
                        csink_angle
                    ),
                });
            }

            // Approximate the countersink cone as a short cylinder at the
            // countersink diameter. The depth of the cone portion is derived
            // from the angle and the diameter difference.
            let half_angle_rad = csink_angle.to_radians() / 2.0;
            let csink_depth =
                (csink_diameter / 2.0 - params.diameter / 2.0) / half_angle_rad.tan();

            // Step 1: Cut the countersink (wider, shallow cylinder approximation).
            let result = cut_cylinder(
                stock,
                params.position,
                params.direction,
                csink_diameter / 2.0,
                csink_depth,
                false,
            )?;

            // Step 2: Cut the main hole below the countersink.
            let dir_norm = params.direction.normalize();
            let deep_center = params.position + dir_norm * csink_depth;
            let remaining_depth = if params.through_all {
                params.depth
            } else {
                params.depth - csink_depth
            };

            if !params.through_all && remaining_depth <= 0.0 {
                return Ok(result);
            }

            cut_cylinder(
                result,
                deep_center,
                params.direction,
                params.diameter / 2.0,
                remaining_depth.max(0.001),
                params.through_all,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::surface::plane::Plane;
    use crate::operations::extrude::{extrude_profile, ExtrudeProfile};
    use crate::topology::body::Body;

    /// Create a 10x10x5 box stock for testing.
    fn make_stock() -> crate::topology::BRep {
        let profile = ExtrudeProfile {
            points: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(10.0, 0.0, 0.0),
                Pt3::new(10.0, 10.0, 0.0),
                Pt3::new(0.0, 10.0, 0.0),
            ],
            plane: Plane::xy(0.0),
        };
        extrude_profile(
            &profile,
            &ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 5.0),
        )
        .unwrap()
    }

    #[test]
    fn simple_hole_in_box() {
        let stock = make_stock();
        let initial_faces = stock.faces.len(); // 6 for a box
        let params = HoleParams {
            hole_type: HoleType::Simple,
            diameter: 2.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        // A blind hole adds CIRCLE_SEGMENTS side-wall faces + 1 cap + 1 inner loop on entry
        // face = initial + CIRCLE_SEGMENTS + 1
        assert!(
            result.faces.len() > initial_faces,
            "Hole should add faces: got {} vs initial {}",
            result.faces.len(),
            initial_faces
        );
    }

    #[test]
    fn counterbore_hole() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Counterbore {
                cbore_diameter: 4.0,
                cbore_depth: 1.0,
            },
            diameter: 2.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        // Two cylindrical cuts produce many faces
        assert!(
            result.faces.len() > 6,
            "Counterbore should produce more than 6 faces: got {}",
            result.faces.len()
        );
    }

    #[test]
    fn countersink_hole() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Countersink {
                csink_diameter: 4.0,
                csink_angle: 82.0,
            },
            diameter: 2.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        assert!(
            result.faces.len() > 6,
            "Countersink should produce more than 6 faces: got {}",
            result.faces.len()
        );
    }

    #[test]
    fn through_all_hole() {
        let stock = make_stock();
        let initial_faces = stock.faces.len();
        let params = HoleParams {
            hole_type: HoleType::Simple,
            diameter: 2.0,
            depth: 1.0, // ignored because through_all
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: true,
        };
        let result = hole_wizard(stock, &params).unwrap();
        assert!(matches!(result.body, Body::Solid(_)));
        // Through-all: both entry and exit faces get inner loops + side walls (no cap)
        assert!(
            result.faces.len() > initial_faces,
            "Through-all hole should add faces: got {} vs initial {}",
            result.faces.len(),
            initial_faces
        );
    }

    #[test]
    fn zero_diameter_fails() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Simple,
            diameter: 0.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params);
        assert!(result.is_err(), "Zero diameter should fail");
    }

    #[test]
    fn negative_diameter_fails() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Simple,
            diameter: -1.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params);
        assert!(result.is_err(), "Negative diameter should fail");
    }

    #[test]
    fn zero_depth_non_through_fails() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Simple,
            diameter: 2.0,
            depth: 0.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params);
        assert!(result.is_err(), "Zero depth (non-through) should fail");
    }

    #[test]
    fn counterbore_smaller_diameter_fails() {
        let stock = make_stock();
        let params = HoleParams {
            hole_type: HoleType::Counterbore {
                cbore_diameter: 1.0, // smaller than hole diameter
                cbore_depth: 1.0,
            },
            diameter: 2.0,
            depth: 3.0,
            position: Pt3::new(5.0, 5.0, 0.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
            through_all: false,
        };
        let result = hole_wizard(stock, &params);
        assert!(
            result.is_err(),
            "Counterbore diameter smaller than hole diameter should fail"
        );
    }
}
