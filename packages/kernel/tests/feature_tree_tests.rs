use blockcad_kernel::feature_tree::*;

fn test_feature(id: &str, name: &str, kind: FeatureKind) -> Feature {
    Feature::new(id.into(), name.into(), kind, FeatureParams::Placeholder)
}

#[test]
fn feature_tree_push_and_cursor() {
    let mut tree = FeatureTree::new();
    tree.push(test_feature("sketch-1", "Sketch1", FeatureKind::Sketch));
    tree.push(test_feature("extrude-1", "Extrude1", FeatureKind::Extrude));
    tree.push(test_feature("fillet-1", "Fillet1", FeatureKind::Fillet));

    assert_eq!(tree.len(), 3);
    assert_eq!(tree.cursor(), Some(2));
    assert_eq!(tree.active_features().len(), 3);
}

#[test]
fn rollback_hides_features() {
    let mut tree = FeatureTree::new();
    tree.push(test_feature("sketch-1", "Sketch1", FeatureKind::Sketch));
    tree.push(test_feature("extrude-1", "Extrude1", FeatureKind::Extrude));
    tree.push(test_feature("fillet-1", "Fillet1", FeatureKind::Fillet));

    tree.rollback_to(2).unwrap();
    assert_eq!(tree.cursor(), Some(1));
    assert_eq!(tree.active_features().len(), 2);
    assert_eq!(tree.active_features()[1].name, "Extrude1");
}

#[test]
fn suppress_and_unsuppress() {
    let mut tree = FeatureTree::new();
    tree.push(test_feature("sketch-1", "Sketch1", FeatureKind::Sketch));
    tree.push(test_feature("extrude-1", "Extrude1", FeatureKind::Extrude));

    tree.suppress(1).unwrap();
    assert!(tree.features()[1].suppressed);
    assert!(!tree.features()[1].is_active());

    tree.unsuppress(1).unwrap();
    assert!(!tree.features()[1].suppressed);
    assert!(tree.features()[1].is_active());
}

#[test]
fn insert_at_cursor_middle() {
    let mut tree = FeatureTree::new();
    tree.push(test_feature("f1", "F1", FeatureKind::Sketch));
    tree.push(test_feature("f3", "F3", FeatureKind::Fillet));

    tree.rollback_to(1).unwrap();
    assert_eq!(tree.cursor(), Some(0));

    let idx = tree
        .insert_at_cursor(test_feature("f2", "F2", FeatureKind::Extrude))
        .unwrap();
    assert_eq!(idx, 1);
    assert_eq!(tree.len(), 3);
    assert_eq!(tree.features()[0].name, "F1");
    assert_eq!(tree.features()[1].name, "F2");
    assert_eq!(tree.features()[2].name, "F3");
}

#[test]
fn dependency_graph_downstream_propagation() {
    use blockcad_kernel::feature_tree::dependency::DependencyGraph;

    let mut graph = DependencyGraph::new();
    graph.add_dependency(1, 0);
    graph.add_dependency(2, 1);
    graph.add_dependency(3, 1);

    let downstream = graph.downstream_of(0);
    assert!(downstream.contains(&1));
    assert!(downstream.contains(&2));
    assert!(downstream.contains(&3));
}
