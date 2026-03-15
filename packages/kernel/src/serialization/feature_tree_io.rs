use crate::error::KernelResult;
use crate::feature_tree::kind::FeatureKind;
use crate::feature_tree::params::FeatureParams;
use crate::feature_tree::FeatureTree;

use super::schema::KernelDocument;

/// Serialize a feature tree into a KernelDocument.
/// Embeds sketch data from `tree.sketches` into feature params so it persists.
pub fn serialize_tree(tree: &FeatureTree, name: &str) -> KernelResult<KernelDocument> {
    let mut features = tree.features().to_vec();
    // Embed sketch data from tree.sketches into feature params
    for (i, feature) in features.iter_mut().enumerate() {
        if feature.kind == FeatureKind::Sketch {
            if let Some(sketch) = tree.sketches.get(&i) {
                feature.params = FeatureParams::Sketch(sketch.clone());
            }
        }
    }
    Ok(KernelDocument::new(name.into(), features))
}

/// Deserialize a KernelDocument into a FeatureTree.
/// Extracts sketch data from params into `tree.sketches` for evaluation.
pub fn deserialize_tree(doc: &KernelDocument) -> KernelResult<FeatureTree> {
    let mut tree = FeatureTree::new();
    for (i, feature) in doc.features.iter().enumerate() {
        tree.push(feature.clone());
        // Extract sketch data from params into tree.sketches
        if let FeatureParams::Sketch(sketch) = &feature.params {
            tree.sketches.insert(i, sketch.clone());
        }
    }
    Ok(tree)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_tree::{Feature, FeatureKind, FeatureParams};

    #[test]
    fn roundtrip_with_sketch_data() {
        use crate::geometry::surface::plane::Plane;
        use crate::geometry::Pt2;
        use crate::sketch::entity::SketchEntity;
        use crate::sketch::constraint::{Constraint, ConstraintKind};
        use crate::sketch::Sketch;

        let mut tree = FeatureTree::new();

        // Add sketch feature
        let mut sketch = Sketch::new(Plane::xy(0.0));
        let p1 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(0.0, 0.0) });
        let p2 = sketch.add_entity(SketchEntity::Point { position: Pt2::new(5.0, 0.0) });
        sketch.add_entity(SketchEntity::Line { start: p1, end: p2 });
        sketch.add_constraint(Constraint::new(ConstraintKind::Fixed, vec![p1]));

        tree.push(Feature::new(
            "sketch-1".into(), "Sketch".into(),
            FeatureKind::Sketch, FeatureParams::Placeholder,
        ));
        tree.sketches.insert(0, sketch);

        // Add extrude feature
        tree.push(Feature::new(
            "extrude-1".into(), "Extrude".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(crate::operations::extrude::ExtrudeParams {
                direction: crate::geometry::Vec3::new(0.0, 0.0, 1.0),
                depth: 5.0, symmetric: false, draft_angle: 0.0,
            }),
        ));

        // Serialize
        let doc = serialize_tree(&tree, "Test Part").unwrap();
        let json = doc.to_json_pretty().unwrap();

        // Verify sketch data is in JSON
        assert!(json.contains("\"type\": \"sketch\""), "Should contain sketch type");

        // Deserialize
        let doc2 = KernelDocument::from_json(&json).unwrap();
        let tree2 = deserialize_tree(&doc2).unwrap();

        assert_eq!(tree2.len(), 2);
        assert!(tree2.sketches.contains_key(&0), "Sketch data should be restored");
        let sketch2 = tree2.sketches.get(&0).unwrap();
        assert_eq!(sketch2.entity_count(), 3); // 2 points + 1 line
        assert_eq!(sketch2.constraint_count(), 1);
    }

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
