use blockcad_kernel::geometry::Pt3;
use blockcad_kernel::topology::*;

#[test]
fn entity_store_generational_safety() {
    let mut store = EntityStore::new();
    let id1 = store.insert(Vertex::new(Pt3::origin()));
    store.remove(id1).unwrap();
    let id2 = store.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));

    // Old ID should not resolve to new entity
    assert!(store.get(id1).is_err());
    assert!(store.get(id2).is_ok());
}

#[test]
fn build_simple_brep() {
    let mut brep = BRep::new();

    // Add 4 vertices of a tetrahedron
    let v0 = brep.vertices.insert(Vertex::new(Pt3::new(0.0, 0.0, 0.0)));
    let v1 = brep.vertices.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));
    let v2 = brep.vertices.insert(Vertex::new(Pt3::new(0.5, 1.0, 0.0)));
    let v3 = brep.vertices.insert(Vertex::new(Pt3::new(0.5, 0.5, 1.0)));

    // 6 edges
    let _e01 = brep.edges.insert(Edge::new(v0, v1));
    let _e02 = brep.edges.insert(Edge::new(v0, v2));
    let _e03 = brep.edges.insert(Edge::new(v0, v3));
    let _e12 = brep.edges.insert(Edge::new(v1, v2));
    let _e13 = brep.edges.insert(Edge::new(v1, v3));
    let _e23 = brep.edges.insert(Edge::new(v2, v3));

    // 4 faces
    brep.faces.insert(Face::new());
    brep.faces.insert(Face::new());
    brep.faces.insert(Face::new());
    brep.faces.insert(Face::new());

    // Euler: V - E + F = 4 - 6 + 4 = 2 (valid closed solid)
    assert_eq!(brep.euler_characteristic(), 2);
    assert_eq!(brep.vertices.len(), 4);
    assert_eq!(brep.edges.len(), 6);
    assert_eq!(brep.faces.len(), 4);
}

#[test]
fn brep_add_and_remove_entities() {
    let mut brep = BRep::new();
    let v1 = brep.vertices.insert(Vertex::new(Pt3::origin()));
    let v2 = brep.vertices.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));
    assert_eq!(brep.vertices.len(), 2);

    brep.vertices.remove(v1).unwrap();
    assert_eq!(brep.vertices.len(), 1);
    assert!(brep.vertices.get(v1).is_err());
    assert!(brep.vertices.get(v2).is_ok());
}

#[test]
fn edge_with_curve() {
    let mut brep = BRep::new();
    let v1 = brep.vertices.insert(Vertex::new(Pt3::origin()));
    let v2 = brep.vertices.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));

    let curve_idx = brep.add_curve(Box::new(
        blockcad_kernel::geometry::curve::line::Line3::new(Pt3::origin(), Pt3::new(1.0, 0.0, 0.0))
            .unwrap(),
    ));

    let edge = Edge::new(v1, v2).with_curve(curve_idx);
    let edge_id = brep.edges.insert(edge);
    let edge = brep.edges.get(edge_id).unwrap();
    assert_eq!(edge.curve_index, Some(0));
}
