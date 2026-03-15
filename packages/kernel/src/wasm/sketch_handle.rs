use wasm_bindgen::prelude::*;

use crate::error::KernelError;
use crate::geometry::surface::plane::Plane;
use crate::geometry::Pt3;
use crate::sketch::Sketch;

/// WASM handle for sketch editing operations.
#[wasm_bindgen]
pub struct SketchHandle {
    sketch: Sketch,
}

#[wasm_bindgen]
impl SketchHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> SketchHandle {
        SketchHandle {
            sketch: Sketch::new(Plane::xy(0.0)),
        }
    }

    /// Number of entities in the sketch
    pub fn entity_count(&self) -> usize {
        self.sketch.entity_count()
    }

    /// Number of constraints in the sketch
    pub fn constraint_count(&self) -> usize {
        self.sketch.constraint_count()
    }

    /// Get all sketch entities as JSON
    pub fn get_entities_json(&self) -> Result<String, JsValue> {
        Err(KernelError::Internal("get_entities_json not yet implemented".into()).into())
    }

    /// Solve constraints and return updated entity positions
    pub fn solve(&mut self) -> Result<String, JsValue> {
        Err(KernelError::Internal("sketch solve not yet implemented".into()).into())
    }
}
