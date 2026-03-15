use wasm_bindgen::prelude::*;

use crate::error::KernelError;
use crate::feature_tree::FeatureTree;
use crate::serialization::schema::KernelDocument;
use crate::serialization::{feature_tree_io, migrations};

/// The main WASM entry point for the kernel.
/// Wraps a FeatureTree and exposes operations to JavaScript.
#[wasm_bindgen]
pub struct KernelHandle {
    tree: FeatureTree,
    name: String,
}

#[wasm_bindgen]
impl KernelHandle {
    #[wasm_bindgen(constructor)]
    pub fn new() -> KernelHandle {
        #[cfg(feature = "wasm")]
        console_error_panic_hook::set_once();
        KernelHandle {
            tree: FeatureTree::new(),
            name: "Untitled".into(),
        }
    }

    pub fn feature_count(&self) -> usize {
        self.tree.len()
    }

    pub fn cursor(&self) -> i32 {
        self.tree.cursor().map(|c| c as i32).unwrap_or(-1)
    }

    /// Add a feature. Returns error if the operation requires server and this is client build.
    pub fn add_feature(&mut self, _kind: &str, _params_json: &str) -> Result<String, JsValue> {
        Err(KernelError::Internal("add_feature not yet implemented".into()).into())
    }

    /// Check if a feature kind is available in this build.
    pub fn is_feature_available(&self, kind: &str) -> bool {
        let _server_only = [
            "boolean_union", "boolean_subtract", "boolean_intersect",
            "sweep", "loft", "shell", "draft",
            "linear_pattern", "circular_pattern", "mirror",
        ];
        #[cfg(feature = "server")]
        return true;
        #[cfg(not(feature = "server"))]
        return !_server_only.contains(&kind);
    }

    pub fn tessellate(
        &mut self,
        _chord_tolerance: f64,
        _angle_tolerance: f64,
    ) -> Result<Vec<u8>, JsValue> {
        Err(KernelError::Internal("tessellate not yet implemented".into()).into())
    }

    /// Serialize to pretty-printed JSON (.blockcad format).
    pub fn serialize(&self) -> Result<String, JsValue> {
        let doc = feature_tree_io::serialize_tree(&self.tree, &self.name)
            .map_err(|e| -> JsValue { e.into() })?;
        doc.to_json_pretty()
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })
    }

    /// Load from a .blockcad JSON document.
    pub fn deserialize(json: &str) -> Result<KernelHandle, JsValue> {
        let doc = KernelDocument::from_json(json)
            .map_err(|e| -> JsValue { KernelError::Serialization(e.to_string()).into() })?;
        let doc = migrations::migrate(doc).map_err(|e| -> JsValue { e.into() })?;
        let tree = feature_tree_io::deserialize_tree(&doc)
            .map_err(|e| -> JsValue { e.into() })?;
        Ok(KernelHandle {
            tree,
            name: doc.metadata.name,
        })
    }

    // --- Server-only operations ---

    #[cfg(feature = "server")]
    pub fn execute_boolean(&mut self, _op: &str, _params_json: &str) -> Result<String, JsValue> {
        Err(KernelError::Internal("boolean ops not yet implemented".into()).into())
    }

    #[cfg(feature = "server")]
    pub fn execute_pattern(&mut self, _kind: &str, _params_json: &str) -> Result<String, JsValue> {
        Err(KernelError::Internal("pattern ops not yet implemented".into()).into())
    }
}
