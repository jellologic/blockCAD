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
        #[cfg(feature = "wasm")]
        console_error_panic_hook::set_once();
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
        let _server_only = [
            "boolean_union",
            "boolean_subtract",
            "boolean_intersect",
            "sweep",
            "loft",
            "shell",
            "draft",
            "linear_pattern",
            "circular_pattern",
            "mirror",
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
