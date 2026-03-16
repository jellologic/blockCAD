//! blockCAD native kernel server — JSON-RPC over stdin/stdout.
//!
//! Spawned as a sidecar process by the Electrobun desktop app.
//! Provides unlimited 64-bit memory for complex geometry operations.
//!
//! Protocol: line-delimited JSON-RPC 2.0 on stdin/stdout.
//! Each request is one line, each response is one line.

use blockcad_kernel::kernel_core::KernelCore;
use blockcad_kernel::tessellation::compute_mass_properties;
use serde_json::{json, Value};
use std::io::{self, BufRead, BufWriter, Write};

fn main() {
    let mut kernel = KernelCore::new();
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        if line.trim().is_empty() {
            continue;
        }

        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                let err = json!({
                    "jsonrpc": "2.0",
                    "id": null,
                    "error": { "code": -32700, "message": format!("Parse error: {}", e) }
                });
                let _ = writeln!(writer, "{}", err);
                let _ = writer.flush();
                continue;
            }
        };

        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let params = request.get("params").cloned().unwrap_or(json!({}));

        let response = match method {
            "ping" => {
                json!({ "jsonrpc": "2.0", "id": id, "result": "pong" })
            }

            "add_feature" => {
                let kind = params.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                let params_json = params.get("params").map(|p| p.to_string()).unwrap_or_default();
                match kernel.add_feature(kind, &params_json) {
                    Ok(feat_id) => json!({ "jsonrpc": "2.0", "id": id, "result": { "id": feat_id } }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "tessellate" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                match kernel.tessellate(chord_tol, angle_tol) {
                    Ok(bytes) => {
                        json!({ "jsonrpc": "2.0", "id": id, "result": { "data": encode_base64(&bytes) } })
                    }
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "get_features" => {
                match kernel.get_features_json() {
                    Ok(json_str) => {
                        let features: Value = serde_json::from_str(&json_str).unwrap_or(json!([]));
                        json!({ "jsonrpc": "2.0", "id": id, "result": features })
                    }
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "serialize" => {
                match kernel.serialize() {
                    Ok(json_str) => json!({ "jsonrpc": "2.0", "id": id, "result": json_str }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "deserialize" => {
                let doc_json = params.get("json").and_then(|j| j.as_str()).unwrap_or("");
                match KernelCore::deserialize(doc_json) {
                    Ok(new_kernel) => {
                        kernel = new_kernel;
                        json!({ "jsonrpc": "2.0", "id": id, "result": "ok" })
                    }
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "suppress" => {
                let index = params.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                match kernel.suppress(index) {
                    Ok(()) => json!({ "jsonrpc": "2.0", "id": id, "result": "ok" }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "unsuppress" => {
                let index = params.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                match kernel.unsuppress(index) {
                    Ok(()) => json!({ "jsonrpc": "2.0", "id": id, "result": "ok" }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "export_stl_binary" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                match kernel.export_stl_binary(chord_tol, angle_tol) {
                    Ok(bytes) => json!({ "jsonrpc": "2.0", "id": id, "result": { "data": encode_base64(&bytes) } }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "export_stl_ascii" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let opts = params.get("options").map(|o| o.to_string()).unwrap_or("{}".into());
                match kernel.export_stl_ascii(chord_tol, angle_tol, &opts) {
                    Ok(text) => json!({ "jsonrpc": "2.0", "id": id, "result": text }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "export_obj" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let opts = params.get("options").map(|o| o.to_string()).unwrap_or("{}".into());
                match kernel.export_obj(chord_tol, angle_tol, &opts) {
                    Ok(text) => json!({ "jsonrpc": "2.0", "id": id, "result": text }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "export_3mf" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let opts = params.get("options").map(|o| o.to_string()).unwrap_or("{}".into());
                match kernel.export_3mf(chord_tol, angle_tol, &opts) {
                    Ok(bytes) => json!({ "jsonrpc": "2.0", "id": id, "result": { "data": encode_base64(&bytes) } }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "export_glb" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let opts = params.get("options").map(|o| o.to_string()).unwrap_or("{}".into());
                match kernel.export_glb(chord_tol, angle_tol, &opts) {
                    Ok(bytes) => json!({ "jsonrpc": "2.0", "id": id, "result": { "data": encode_base64(&bytes) } }),
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "mass_properties" => {
                let chord_tol = params.get("chord_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.01);
                let angle_tol = params.get("angle_tolerance").and_then(|v| v.as_f64()).unwrap_or(0.5);
                match kernel.build_mesh(chord_tol, angle_tol) {
                    Ok(mesh) => {
                        let props = compute_mass_properties(&mesh);
                        json!({ "jsonrpc": "2.0", "id": id, "result": props })
                    }
                    Err(e) => make_error(&id, -1, &e.to_string()),
                }
            }

            "feature_count" => {
                json!({ "jsonrpc": "2.0", "id": id, "result": kernel.feature_count() })
            }

            _ => {
                make_error(&id, -32601, &format!("Method not found: {}", method))
            }
        };

        let _ = writeln!(writer, "{}", response);
        let _ = writer.flush();
    }
}

fn make_error(id: &Value, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message }
    })
}

fn encode_base64(data: &[u8]) -> String {
    // Simple base64 encoding without external crate
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
