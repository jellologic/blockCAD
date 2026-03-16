//! Assembly document serialization — .blockcad-assembly JSON format.

use crate::assembly::{Assembly, Component, Mate, Part};
use crate::error::KernelResult;

use super::feature_tree_io;
use super::schema::{KernelDocument, Metadata, SCHEMA_VERSION};

/// Serializable assembly document containing parts and component instances.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyDocument {
    #[serde(rename = "$schema", default, skip_serializing_if = "Option::is_none")]
    pub schema_url: Option<String>,
    pub version: u32,
    pub metadata: Metadata,
    /// Embedded part documents.
    pub parts: Vec<KernelDocument>,
    /// Component instances with transforms.
    pub components: Vec<Component>,
    /// Mate constraints between components.
    pub mates: Vec<Mate>,
    /// Explosion steps for exploded views.
    #[serde(default)]
    pub explosion_steps: Vec<crate::assembly::ExplosionStep>,
}

impl AssemblyDocument {
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Serialize an Assembly to an AssemblyDocument.
pub fn serialize_assembly(assembly: &Assembly, name: &str) -> KernelResult<AssemblyDocument> {
    let mut parts = Vec::new();
    for part in &assembly.parts {
        let doc = feature_tree_io::serialize_tree(&part.tree, &part.name)?;
        parts.push(doc);
    }

    Ok(AssemblyDocument {
        schema_url: Some("https://blockcad.dev/schema/assembly/v1.json".into()),
        version: SCHEMA_VERSION,
        metadata: Metadata::new(name.into()),
        parts,
        components: assembly.components.clone(),
        mates: assembly.mates.clone(),
        explosion_steps: assembly.explosion_steps.clone(),
    })
}

/// Deserialize an AssemblyDocument into an Assembly.
pub fn deserialize_assembly(doc: &AssemblyDocument) -> KernelResult<Assembly> {
    let mut parts = Vec::new();
    for (i, part_doc) in doc.parts.iter().enumerate() {
        let tree = feature_tree_io::deserialize_tree(part_doc)?;
        parts.push(Part {
            id: format!("part-{}", i),
            name: part_doc.metadata.name.clone(),
            tree,
        });
    }

    Ok(Assembly {
        parts,
        components: doc.components.clone(),
        mates: doc.mates.clone(),
        explosion_steps: doc.explosion_steps.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams, FeatureTree};
    use crate::geometry::Vec3;
    use crate::geometry::transform;
    use crate::operations::extrude::ExtrudeParams;

    fn make_simple_part(id: &str, name: &str) -> Part {
        let mut tree = FeatureTree::new();
        tree.push(Feature::new(
            "e1".into(), "Extrude".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(ExtrudeParams::blind(Vec3::new(0.0, 0.0, 1.0), 10.0)),
        ));
        Part { id: id.into(), name: name.into(), tree }
    }

    #[test]
    fn assembly_document_roundtrip() {
        let mut assembly = Assembly::new();
        assembly.add_part(make_simple_part("part1", "Box A"));
        assembly.add_component(
            Component::new("comp1".into(), "part1".into(), "Instance 1".into())
                .with_transform(transform::translation(10.0, 0.0, 0.0))
        );

        let doc = serialize_assembly(&assembly, "Test Assembly").unwrap();
        let json = doc.to_json_pretty().unwrap();

        assert!(json.contains("Test Assembly"));
        assert!(json.contains("comp1"));

        let doc2 = AssemblyDocument::from_json(&json).unwrap();
        assert_eq!(doc2.parts.len(), 1);
        assert_eq!(doc2.components.len(), 1);
        assert_eq!(doc2.components[0].id, "comp1");
    }
}
