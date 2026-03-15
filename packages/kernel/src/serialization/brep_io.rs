use crate::error::{KernelError, KernelResult};
use crate::topology::BRep;

/// Serialize a BRep to a JSON-compatible schema.
pub fn serialize_brep(_brep: &BRep) -> KernelResult<serde_json::Value> {
    // TODO: Walk all entities and serialize to a structured format
    Err(KernelError::Internal("BRep serialization not yet implemented".into()))
}

/// Deserialize a BRep from a JSON-compatible schema.
pub fn deserialize_brep(_value: &serde_json::Value) -> KernelResult<BRep> {
    Err(KernelError::Internal("BRep deserialization not yet implemented".into()))
}
