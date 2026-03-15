use crate::error::KernelResult;
use crate::feature_tree::FeatureTree;

use super::schema::KernelDocument;

/// Serialize a feature tree into a KernelDocument.
pub fn serialize_tree(tree: &FeatureTree, name: &str) -> KernelResult<KernelDocument> {
    Ok(KernelDocument::new(name.into(), tree.features().to_vec()))
}

/// Deserialize a KernelDocument into a FeatureTree.
pub fn deserialize_tree(doc: &KernelDocument) -> KernelResult<FeatureTree> {
    let mut tree = FeatureTree::new();
    for feature in &doc.features {
        tree.push(feature.clone());
    }
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams};

    #[test]
    fn roundtrip_feature_tree() {
        let mut tree = FeatureTree::new();
        tree.push(Feature::new(
            "extrude-1".into(),
            "F1".into(),
            FeatureKind::Extrude,
            FeatureParams::Placeholder,
        ));
        tree.push(Feature::new(
            "fillet-1".into(),
            "F2".into(),
            FeatureKind::Fillet,
            FeatureParams::Placeholder,
        ));

        let doc = serialize_tree(&tree, "Test Part").unwrap();
        let json = doc.to_json_pretty().unwrap();
        let doc2 = KernelDocument::from_json(&json).unwrap();
        let tree2 = deserialize_tree(&doc2).unwrap();

        assert_eq!(tree2.len(), 2);
        assert_eq!(tree2.features()[0].name, "F1");
        assert_eq!(tree2.features()[1].name, "F2");
    }
}
