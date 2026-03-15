use wasm_bindgen::JsValue;

use crate::error::KernelError;

impl From<KernelError> for JsValue {
    fn from(e: KernelError) -> Self {
        let json = serde_json::json!({
            "kind": e.kind_str(),
            "message": e.to_string(),
        });
        JsValue::from_str(&json.to_string())
    }
}
