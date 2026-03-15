use wasm_bindgen::prelude::*;

use crate::error::KernelError;
use crate::geometry::surface::plane::Plane;
use crate::geometry::{Pt2, Pt3, Vec3};
use crate::sketch::constraint::{Constraint, ConstraintKind};
use crate::sketch::entity::{SketchEntity, SketchEntityId};
use crate::sketch::solver_bridge::build_constraint_graph;
use crate::sketch::Sketch;
use crate::solver::newton_raphson::{solve, SolverConfig};

/// WASM handle for sketch editing operations.
/// Provides real-time constraint solving via the Rust solver.
#[wasm_bindgen]
pub struct SketchHandle {
    sketch: Sketch,
}

/// JSON representations for sketch entities passed from JavaScript
#[derive(serde::Deserialize)]
struct PlaneJson {
    origin: [f64; 3],
    normal: [f64; 3],
    #[serde(rename = "uAxis")]
    u_axis: [f64; 3],
    #[serde(rename = "vAxis")]
    v_axis: [f64; 3],
}

#[derive(serde::Deserialize)]
#[serde(tag = "type")]
enum EntityJson {
    #[serde(rename = "point")]
    Point { x: f64, y: f64 },
    #[serde(rename = "line")]
    Line {
        #[serde(rename = "startIndex")]
        start_index: u32,
        #[serde(rename = "endIndex")]
        end_index: u32,
    },
    #[serde(rename = "circle")]
    Circle {
        #[serde(rename = "centerIndex")]
        center_index: u32,
        radius: f64,
    },
    #[serde(rename = "arc")]
    Arc {
        #[serde(rename = "centerIndex")]
        center_index: u32,
        #[serde(rename = "startIndex")]
        start_index: u32,
        #[serde(rename = "endIndex")]
        end_index: u32,
    },
}

#[derive(serde::Deserialize)]
struct ConstraintJson {
    kind: String,
    #[serde(rename = "entityIndices")]
    entity_indices: Vec<u32>,
    value: Option<f64>,
}

/// Result returned by solve() — entity positions after solving
#[derive(serde::Serialize)]
struct SolveResult {
    converged: bool,
    iterations: usize,
    entities: Vec<SolvedEntity>,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum SolvedEntity {
    #[serde(rename = "point")]
    Point { x: f64, y: f64 },
    #[serde(rename = "line")]
    Line {},
    #[serde(rename = "circle")]
    Circle { radius: f64 },
    #[serde(rename = "arc")]
    Arc {},
}

#[wasm_bindgen]
impl SketchHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SketchHandle {
        SketchHandle {
            sketch: Sketch::new(Plane::xy(0.0)),
        }
    }

    /// Create a sketch on a specified plane (JSON: {origin, normal, uAxis, vAxis})
    pub fn new_on_plane(plane_json: &str) -> Result<SketchHandle, JsValue> {
        let pj: PlaneJson = serde_json::from_str(plane_json)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })?;
        let plane = Plane {
            origin: Pt3::new(pj.origin[0], pj.origin[1], pj.origin[2]),
            normal: Vec3::new(pj.normal[0], pj.normal[1], pj.normal[2]),
            u_axis: Vec3::new(pj.u_axis[0], pj.u_axis[1], pj.u_axis[2]),
            v_axis: Vec3::new(pj.v_axis[0], pj.v_axis[1], pj.v_axis[2]),
        };
        Ok(SketchHandle {
            sketch: Sketch::new(plane),
        })
    }

    /// Number of entities in the sketch
    pub fn entity_count(&self) -> usize {
        self.sketch.entity_count()
    }

    /// Number of constraints in the sketch
    pub fn constraint_count(&self) -> usize {
        self.sketch.constraint_count()
    }

    /// Add a sketch entity. Returns the entity index.
    /// JSON format: {"type":"point","x":0,"y":0} or {"type":"line","startIndex":0,"endIndex":1}
    pub fn add_entity(&mut self, entity_json: &str) -> Result<u32, JsValue> {
        let ej: EntityJson = serde_json::from_str(entity_json)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })?;

        let entity = match ej {
            EntityJson::Point { x, y } => SketchEntity::Point {
                position: Pt2::new(x, y),
            },
            EntityJson::Line {
                start_index,
                end_index,
            } => SketchEntity::Line {
                start: SketchEntityId::new(start_index, 0),
                end: SketchEntityId::new(end_index, 0),
            },
            EntityJson::Circle {
                center_index,
                radius,
            } => SketchEntity::Circle {
                center: SketchEntityId::new(center_index, 0),
                radius,
            },
            EntityJson::Arc {
                center_index,
                start_index,
                end_index,
            } => SketchEntity::Arc {
                center: SketchEntityId::new(center_index, 0),
                start: SketchEntityId::new(start_index, 0),
                end: SketchEntityId::new(end_index, 0),
            },
        };

        let id = self.sketch.add_entity(entity);
        Ok(id.index())
    }

    /// Add a constraint.
    /// JSON: {"kind":"horizontal","entityIndices":[4],"value":null}
    pub fn add_constraint(&mut self, constraint_json: &str) -> Result<u32, JsValue> {
        let cj: ConstraintJson = serde_json::from_str(constraint_json)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })?;

        let entity_ids: Vec<SketchEntityId> = cj
            .entity_indices
            .iter()
            .map(|&idx| SketchEntityId::new(idx, 0))
            .collect();

        let kind = match cj.kind.as_str() {
            "coincident" => ConstraintKind::Coincident,
            "horizontal" => ConstraintKind::Horizontal,
            "vertical" => ConstraintKind::Vertical,
            "fixed" => ConstraintKind::Fixed,
            "parallel" => ConstraintKind::Parallel,
            "perpendicular" => ConstraintKind::Perpendicular,
            "collinear" => ConstraintKind::Collinear,
            "midpoint" => ConstraintKind::Midpoint,
            "equal" => ConstraintKind::Equal,
            "tangent" => ConstraintKind::Tangent,
            "distance" => ConstraintKind::Distance {
                value: cj.value.unwrap_or(0.0),
            },
            "angle" => ConstraintKind::Angle {
                value: cj.value.unwrap_or(0.0),
                supplementary: false,
            },
            "radius" => ConstraintKind::Radius {
                value: cj.value.unwrap_or(0.0),
            },
            "diameter" => ConstraintKind::Diameter {
                value: cj.value.unwrap_or(0.0),
            },
            other => {
                return Err(KernelError::InvalidParameter {
                    param: "kind".into(),
                    value: other.into(),
                }
                .into());
            }
        };

        let constraint = Constraint::new(kind, entity_ids);
        let id = self.sketch.add_constraint(constraint);
        Ok(id.index())
    }

    /// Solve constraints and return solved entity positions as JSON.
    /// Returns: {"converged":true,"iterations":5,"entities":[{"type":"point","x":0,"y":0},...]}
    pub fn solve(&mut self) -> Result<String, JsValue> {
        let (mut graph, var_map) =
            build_constraint_graph(&self.sketch).map_err(|e| -> JsValue { e.into() })?;

        let result = solve(&mut graph, &SolverConfig::default())
            .map_err(|e| -> JsValue { e.into() })?;

        // Read solved positions back and update sketch entities
        let mut solved_entities = Vec::new();
        for (entity_id, entity) in self.sketch.entities.iter() {
            match entity {
                SketchEntity::Point { .. } => {
                    if let Some((xv, yv)) = var_map.point_vars(entity_id) {
                        let x = graph.variables.value(xv);
                        let y = graph.variables.value(yv);
                        solved_entities.push(SolvedEntity::Point { x, y });
                    }
                }
                SketchEntity::Line { .. } => {
                    solved_entities.push(SolvedEntity::Line {});
                }
                SketchEntity::Circle { .. } => {
                    if let Some(rv) = var_map.circle_radius_var(entity_id) {
                        let radius = graph.variables.value(rv);
                        solved_entities.push(SolvedEntity::Circle { radius });
                    } else {
                        solved_entities.push(SolvedEntity::Circle {
                            radius: match entity {
                                SketchEntity::Circle { radius, .. } => *radius,
                                _ => 0.0,
                            },
                        });
                    }
                }
                SketchEntity::Arc { .. } => {
                    solved_entities.push(SolvedEntity::Arc {});
                }
                SketchEntity::Spline { .. } => {
                    // Not yet supported in solver
                }
            }
        }

        // Write solved values back into the sketch entities
        for (entity_id, entity) in self.sketch.entities.iter() {
            if let SketchEntity::Point { .. } = entity {
                if let Some((xv, yv)) = var_map.point_vars(entity_id) {
                    let x = graph.variables.value(xv);
                    let y = graph.variables.value(yv);
                    // We need mutable access — do it in a second pass
                    // Store updates to apply after iteration
                    let _ = (entity_id, x, y);
                }
            }
        }

        // Second pass: update entity positions in sketch
        let mut updates: Vec<(SketchEntityId, f64, f64)> = Vec::new();
        for (entity_id, entity) in self.sketch.entities.iter() {
            if let SketchEntity::Point { .. } = entity {
                if let Some((xv, yv)) = var_map.point_vars(entity_id) {
                    updates.push((entity_id, graph.variables.value(xv), graph.variables.value(yv)));
                }
            }
        }
        for (entity_id, x, y) in updates {
            if let Ok(entity) = self.sketch.entities.get_mut(entity_id) {
                if let SketchEntity::Point { ref mut position } = entity {
                    position.x = x;
                    position.y = y;
                }
            }
        }

        let solve_result = SolveResult {
            converged: result.converged,
            iterations: result.iterations,
            entities: solved_entities,
        };

        serde_json::to_string(&solve_result)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })
    }

    /// Get all sketch entities as JSON array
    pub fn get_entities_json(&self) -> Result<String, JsValue> {
        let mut entities = Vec::new();
        for (_entity_id, entity) in self.sketch.entities.iter() {
            match entity {
                SketchEntity::Point { position } => {
                    entities.push(SolvedEntity::Point {
                        x: position.x,
                        y: position.y,
                    });
                }
                SketchEntity::Line { .. } => {
                    entities.push(SolvedEntity::Line {});
                }
                SketchEntity::Circle { radius, .. } => {
                    entities.push(SolvedEntity::Circle { radius: *radius });
                }
                SketchEntity::Arc { .. } => {
                    entities.push(SolvedEntity::Arc {});
                }
                SketchEntity::Spline { .. } => {}
            }
        }
        serde_json::to_string(&entities)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })
    }

    /// Update a point entity's position (for dragging). Takes entity index, new x, new y.
    pub fn update_point(&mut self, entity_index: u32, x: f64, y: f64) -> Result<(), JsValue> {
        let entity_id = SketchEntityId::new(entity_index, 0);
        let entity = self
            .sketch
            .entities
            .get_mut(entity_id)
            .map_err(|e| -> JsValue { e.into() })?;
        if let SketchEntity::Point { ref mut position } = entity {
            position.x = x;
            position.y = y;
            Ok(())
        } else {
            Err(KernelError::InvalidParameter {
                param: "entity_index".into(),
                value: "not a point entity".into(),
            }
            .into())
        }
    }
}
