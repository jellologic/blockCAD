use crate::geometry::curve::Curve;
use crate::geometry::surface::Surface;

use super::*;

/// The top-level B-Rep (Boundary Representation) data structure.
/// Contains all topological entities and their associated geometry.
#[derive(Debug)]
pub struct BRep {
    pub vertices: EntityStore<Vertex>,
    pub edges: EntityStore<Edge>,
    pub coedges: EntityStore<CoEdge>,
    pub loops: EntityStore<Loop>,
    pub faces: EntityStore<Face>,
    pub shells: EntityStore<Shell>,
    pub solids: EntityStore<Solid>,

    /// Curve geometry referenced by edges via curve_index
    pub curves: Vec<Box<dyn Curve>>,
    /// Surface geometry referenced by faces via surface_index
    pub surfaces: Vec<Box<dyn Surface>>,

    pub body: Body,
}

impl BRep {
    pub fn new() -> Self {
        Self {
            vertices: EntityStore::new(),
            edges: EntityStore::new(),
            coedges: EntityStore::new(),
            loops: EntityStore::new(),
            faces: EntityStore::new(),
            shells: EntityStore::new(),
            solids: EntityStore::new(),
            curves: Vec::new(),
            surfaces: Vec::new(),
            body: Body::Empty,
        }
    }

    /// Add a curve and return its index
    pub fn add_curve(&mut self, curve: Box<dyn Curve>) -> usize {
        let index = self.curves.len();
        self.curves.push(curve);
        index
    }

    /// Add a surface and return its index
    pub fn add_surface(&mut self, surface: Box<dyn Surface>) -> usize {
        let index = self.surfaces.len();
        self.surfaces.push(surface);
        index
    }

    /// Euler characteristic: V - E + F
    /// For a valid closed solid, this should equal 2 (sphere topology).
    pub fn euler_characteristic(&self) -> i64 {
        let v = self.vertices.len() as i64;
        let e = self.edges.len() as i64;
        let f = self.faces.len() as i64;
        v - e + f
    }

    /// Count of all topological entities
    pub fn entity_count(&self) -> usize {
        self.vertices.len()
            + self.edges.len()
            + self.coedges.len()
            + self.loops.len()
            + self.faces.len()
            + self.shells.len()
            + self.solids.len()
    }
}

impl Clone for BRep {
    fn clone(&self) -> Self {
        Self {
            vertices: self.vertices.clone(),
            edges: self.edges.clone(),
            coedges: self.coedges.clone(),
            loops: self.loops.clone(),
            faces: self.faces.clone(),
            shells: self.shells.clone(),
            solids: self.solids.clone(),
            curves: self.curves.clone(),
            surfaces: self.surfaces.clone(),
            body: self.body.clone(),
        }
    }
}

impl Default for BRep {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Pt3;

    #[test]
    fn empty_brep() {
        let brep = BRep::new();
        assert_eq!(brep.entity_count(), 0);
        assert_eq!(brep.euler_characteristic(), 0);
    }

    #[test]
    fn add_vertices() {
        let mut brep = BRep::new();
        let v1 = brep.vertices.insert(Vertex::new(Pt3::origin()));
        let v2 = brep.vertices.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));
        assert_eq!(brep.vertices.len(), 2);
        assert_eq!(brep.vertices.get(v1).unwrap().point, Pt3::origin());
        assert_eq!(
            brep.vertices.get(v2).unwrap().point,
            Pt3::new(1.0, 0.0, 0.0)
        );
    }

    #[test]
    fn add_edge_between_vertices() {
        let mut brep = BRep::new();
        let v1 = brep.vertices.insert(Vertex::new(Pt3::origin()));
        let v2 = brep.vertices.insert(Vertex::new(Pt3::new(1.0, 0.0, 0.0)));
        let _e = brep.edges.insert(Edge::new(v1, v2));
        assert_eq!(brep.edges.len(), 1);
    }

    #[test]
    fn euler_characteristic_tetrahedron() {
        // Tetrahedron: V=4, E=6, F=4 → χ=2
        let mut brep = BRep::new();
        for _ in 0..4 {
            brep.vertices.insert(Vertex::new(Pt3::origin()));
        }
        let vids: Vec<VertexId> = brep.vertices.iter().map(|(id, _)| id).collect();
        // 6 edges
        let pairs = [(0, 1), (0, 2), (0, 3), (1, 2), (1, 3), (2, 3)];
        for (a, b) in pairs {
            brep.edges.insert(Edge::new(vids[a], vids[b]));
        }
        // 4 faces
        for _ in 0..4 {
            brep.faces.insert(Face::new());
        }
        assert_eq!(brep.euler_characteristic(), 2);
    }
}
