use blockcad_kernel::feature_tree::*;
use blockcad_kernel::serialization::feature_tree_io;
use blockcad_kernel::serialization::migrations;
use blockcad_kernel::serialization::schema::{KernelDocument, Metadata, SCHEMA_VERSION};

#[test]
fn feature_tree_roundtrip() {
    let mut tree = FeatureTree::new();
    tree.push(Feature::new(
        "sketch-1".into(),
        "Sketch1".into(),
        FeatureKind::Sketch,
        FeatureParams::Placeholder,
    ));
    tree.push(Feature::new(
        "extrude-1".into(),
        "Extrude1".into(),
        FeatureKind::Extrude,
        FeatureParams::Placeholder,
    ));

    let doc = feature_tree_io::serialize_tree(&tree, "Test Part").unwrap();
    let json = doc.to_json_pretty().unwrap();
    let doc2 = KernelDocument::from_json(&json).unwrap();
    let tree2 = feature_tree_io::deserialize_tree(&doc2).unwrap();

    assert_eq!(tree2.len(), tree.len());
    for (i, (orig, restored)) in tree.features().iter().zip(tree2.features()).enumerate() {
        assert_eq!(orig.name, restored.name, "Feature {} name mismatch", i);
        assert_eq!(orig.kind, restored.kind, "Feature {} kind mismatch", i);
        assert_eq!(orig.id, restored.id, "Feature {} id mismatch", i);
    }
}

#[test]
fn schema_version_is_one() {
    assert_eq!(SCHEMA_VERSION, 1);
}

#[test]
fn migration_noop_for_current_version() {
    let doc = KernelDocument::new("Test".into(), vec![]);
    let result = migrations::migrate(doc);
    assert!(result.is_ok());
    assert_eq!(result.unwrap().version, SCHEMA_VERSION);
}

#[test]
fn migration_rejects_future_version() {
    let doc = KernelDocument {
        schema_url: None,
        version: 999,
        metadata: Metadata::default(),
        features: vec![],
    };
    assert!(migrations::migrate(doc).is_err());
}

#[test]
fn feature_params_adjacently_tagged() {
    use blockcad_kernel::geometry::Vec3;
    use blockcad_kernel::operations::extrude::ExtrudeParams;

    let params = FeatureParams::Extrude(ExtrudeParams {
        direction: Vec3::new(0.0, 0.0, 1.0),
        depth: 10.0,
        symmetric: false,
        draft_angle: 0.0,
    });

    let json = serde_json::to_string(&params).unwrap();
    // Verify adjacently-tagged format
    assert!(json.contains(r#""type":"extrude""#));
    assert!(json.contains(r#""params""#));

    let restored: FeatureParams = serde_json::from_str(&json).unwrap();
    if let FeatureParams::Extrude(p) = restored {
        assert_eq!(p.depth, 10.0);
    } else {
        panic!("Expected Extrude params");
    }
}

#[test]
fn json_golden_format() {
    use blockcad_kernel::geometry::Vec3;
    use blockcad_kernel::operations::extrude::ExtrudeParams;

    let doc = KernelDocument::new(
        "Test Part".into(),
        vec![Feature::new(
            "extrude-1".into(),
            "Base Extrude".into(),
            FeatureKind::Extrude,
            FeatureParams::Extrude(ExtrudeParams {
                direction: Vec3::new(0.0, 0.0, 1.0),
                depth: 25.0,
                symmetric: false,
                draft_angle: 0.0,
            }),
        )],
    );

    let json = doc.to_json_pretty().unwrap();

    // Schema and version at top level
    assert!(json.contains(r#""$schema""#));
    assert!(json.contains(r#""version": 1"#));

    // Metadata present
    assert!(json.contains(r#""name": "Test Part""#));

    // Feature uses snake_case type
    assert!(json.contains(r#""type": "extrude""#));

    // Feature has id
    assert!(json.contains(r#""id": "extrude-1""#));

    // Suppressed flag (not transient state)
    assert!(json.contains(r#""suppressed": false"#));
    assert!(!json.contains("Pending"));
    assert!(!json.contains("\"state\""));
}

#[test]
fn suppressed_feature_persists() {
    let mut feature = Feature::new(
        "fillet-1".into(),
        "Edge Fillet".into(),
        FeatureKind::Fillet,
        FeatureParams::Placeholder,
    );
    feature.suppressed = true;

    let json = serde_json::to_string(&feature).unwrap();
    assert!(json.contains(r#""suppressed":true"#));

    let restored: Feature = serde_json::from_str(&json).unwrap();
    assert!(restored.suppressed);
    assert!(!restored.is_active());
}

#[test]
fn feature_kind_snake_case() {
    assert_eq!(
        serde_json::to_string(&FeatureKind::BooleanUnion).unwrap(),
        r#""boolean_union""#
    );
    assert_eq!(
        serde_json::to_string(&FeatureKind::LinearPattern).unwrap(),
        r#""linear_pattern""#
    );
    assert_eq!(
        serde_json::to_string(&FeatureKind::Extrude).unwrap(),
        r#""extrude""#
    );
}

#[test]
fn server_params_deserialize_on_any_build() {
    let json = r#"{"type":"sweep","params":{"path_curve_index":0,"twist":0.5}}"#;
    let p: FeatureParams = serde_json::from_str(json).unwrap();
    if let FeatureParams::Sweep(v) = &p {
        assert_eq!(v["path_curve_index"], 0);
    } else {
        panic!("Expected Sweep variant");
    }
}
