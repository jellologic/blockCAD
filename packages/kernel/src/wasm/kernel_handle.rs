use wasm_bindgen::prelude::*;

use crate::kernel_core::KernelCore;

/// The main WASM entry point for the kernel.
/// Delegates to KernelCore for all operations.
#[wasm_bindgen]
pub struct KernelHandle {
    core: KernelCore,
}

#[wasm_bindgen]
impl KernelHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> KernelHandle {
        // Note: console_error_panic_hook removed to avoid re-entrant JS calls
        // that cause "recursive use of an object" errors in wasm-bindgen
        KernelHandle {
            core: KernelCore::new(),
        }
    }

    pub fn feature_count(&self) -> usize {
        self.core.feature_count()
    }

    pub fn cursor(&self) -> i32 {
        self.core.cursor().map(|c| c as i32).unwrap_or(-1)
    }

    /// Add a feature. Returns the feature ID on success.
    pub fn add_feature(&mut self, kind: &str, params_json: &str) -> Result<String, JsValue> {
        self.core
            .add_feature(kind, params_json)
            .map_err(|e| e.into())
    }

    /// Check if a feature kind is available in this build.
    pub fn is_feature_available(&self, kind: &str) -> bool {
        // LinearPattern, CircularPattern, Mirror, and Shell are now client operations.
        let _server_only = [
            "boolean_union",
            "boolean_subtract",
            "boolean_intersect",
            "sweep",
            "loft",
            "draft",
        ];
        #[cfg(feature = "server")]
        return true;
        #[cfg(not(feature = "server"))]
        return !_server_only.contains(&kind);
    }

    /// Tessellate the current model state into a byte buffer.
    pub fn tessellate(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> Result<Vec<u8>, JsValue> {
        self.core
            .tessellate(chord_tolerance, angle_tolerance)
            .map_err(|e| e.into())
    }

    /// Get the feature list as JSON.
    pub fn get_features_json(&self) -> Result<String, JsValue> {
        self.core.get_features_json().map_err(|e| e.into())
    }

    /// Serialize to pretty-printed JSON (.blockcad format).
    pub fn serialize(&self) -> Result<String, JsValue> {
        self.core.serialize().map_err(|e| e.into())
    }

    /// Load from a .blockcad JSON document.
    pub fn deserialize(json: &str) -> Result<KernelHandle, JsValue> {
        let core = KernelCore::deserialize(json).map_err(|e| -> JsValue { e.into() })?;
        Ok(KernelHandle { core })
    }

    /// Suppress a feature by index.
    pub fn suppress(&mut self, index: usize) -> Result<(), JsValue> {
        self.core.suppress(index).map_err(|e| e.into())
    }

    /// Unsuppress a feature by index.
    pub fn unsuppress(&mut self, index: usize) -> Result<(), JsValue> {
        self.core.unsuppress(index).map_err(|e| e.into())
    }

    // --- Export operations ---

    /// Export as binary STL bytes.
    pub fn export_stl_binary(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
    ) -> Result<Vec<u8>, JsValue> {
        self.core
            .export_stl_binary(chord_tolerance, angle_tolerance)
            .map_err(|e| e.into())
    }

    /// Export as ASCII STL string.
    pub fn export_stl_ascii(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .export_stl_ascii(chord_tolerance, angle_tolerance, options_json)
            .map_err(|e| e.into())
    }

    /// Export as Wavefront OBJ string.
    pub fn export_obj(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .export_obj(chord_tolerance, angle_tolerance, options_json)
            .map_err(|e| e.into())
    }

    /// Export as 3MF bytes (ZIP archive).
    pub fn export_3mf(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<Vec<u8>, JsValue> {
        self.core
            .export_3mf(chord_tolerance, angle_tolerance, options_json)
            .map_err(|e| e.into())
    }

    /// Export as GLB (binary glTF 2.0) bytes.
    pub fn export_glb(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<Vec<u8>, JsValue> {
        self.core
            .export_glb(chord_tolerance, angle_tolerance, options_json)
            .map_err(|e| e.into())
    }

    /// Export as STEP (ISO 10303-21) string.
    pub fn export_step(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        options_json: &str,
    ) -> Result<String, JsValue> {
        self.core
            .export_step(chord_tolerance, angle_tolerance, options_json)
            .map_err(|e| e.into())
    }

    /// Compute mass properties (volume, surface area, center of mass, inertia tensor).
    /// Returns JSON-serialized MassProperties. If density > 0, inertia is scaled.
    pub fn compute_mass_properties(
        &mut self,
        chord_tolerance: f64,
        angle_tolerance: f64,
        density: f64,
    ) -> Result<String, JsValue> {
        let density_opt = if density > 0.0 { Some(density) } else { None };
        let props = self.core
            .compute_mass_properties(chord_tolerance, angle_tolerance, density_opt)
            .map_err(|e| -> JsValue { e.into() })?;
        serde_json::to_string(&props)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Evaluate the feature tree and return cache metrics as JSON.
    /// Returns `{"features_evaluated": N, "features_skipped_param_hash": N, "features_skipped_fingerprint": N}`
    pub fn evaluate_with_metrics(&mut self) -> Result<String, JsValue> {
        let metrics = self.core.evaluate_with_metrics().map_err(|e| -> JsValue { e.into() })?;
        let json = format!(
            r#"{{"features_evaluated":{},"features_skipped_param_hash":{},"features_skipped_fingerprint":{}}}"#,
            metrics.features_evaluated,
            metrics.features_skipped_param_hash,
            metrics.features_skipped_fingerprint,
        );
        Ok(json)
    }

    // --- Server-only operations ---

    #[cfg(feature = "server")]
    pub fn execute_boolean(&mut self, _op: &str, _params_json: &str) -> Result<String, JsValue> {
        Err(crate::error::KernelError::Internal("boolean ops not yet implemented".into()).into())
    }

    #[cfg(feature = "server")]
    pub fn execute_pattern(
        &mut self,
        _kind: &str,
        _params_json: &str,
    ) -> Result<String, JsValue> {
        Err(crate::error::KernelError::Internal("pattern ops not yet implemented".into()).into())
    }
}
