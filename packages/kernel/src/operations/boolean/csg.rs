//! BSP-tree CSG engine for Boolean operations on planar-face BRep solids.
//!
//! Based on the BSP-CSG algorithm: builds Binary Space Partition trees from
//! planar polygons, clips polygons against trees, and selects fragments
//! based on the desired Boolean operation (union, subtract, intersect).

use crate::error::KernelResult;
use crate::geometry::{Pt3, Vec3};
use crate::topology::BRep;
use crate::topology::builders::{extract_face_polygons, rebuild_brep_from_faces};

const EPSILON: f64 = 1e-5;

// ─── Core Types ────────────────────────────────────────────────

/// A splitting plane defined by normal and signed distance from origin.
#[derive(Debug, Clone)]
pub struct CsgPlane {
    pub normal: Vec3,
    pub w: f64, // dot(normal, any_point_on_plane)
}

impl CsgPlane {
    pub fn from_points(a: &Pt3, b: &Pt3, c: &Pt3) -> Option<Self> {
        let ab = b - a;
        let ac = c - a;
        let n = Vec3::new(
            ab.y * ac.z - ab.z * ac.y,
            ab.z * ac.x - ab.x * ac.z,
            ab.x * ac.y - ab.y * ac.x,
        );
        let len = n.norm();
        if len < 1e-12 {
            return None;
        }
        let normal = n / len;
        let w = normal.dot(&Vec3::new(a.x, a.y, a.z));
        Some(CsgPlane { normal, w })
    }

    pub fn from_normal_and_point(normal: &Vec3, point: &Pt3) -> Self {
        let n = normal.normalize();
        CsgPlane {
            w: n.dot(&Vec3::new(point.x, point.y, point.z)),
            normal: n,
        }
    }

    /// Signed distance from point to plane. Positive = front, negative = back.
    pub fn signed_distance(&self, p: &Pt3) -> f64 {
        self.normal.dot(&Vec3::new(p.x, p.y, p.z)) - self.w
    }
}

/// Classification of a point relative to a plane.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Side {
    Front,
    Back,
    Coplanar,
}

fn classify_point(plane: &CsgPlane, p: &Pt3) -> Side {
    let d = plane.signed_distance(p);
    if d > EPSILON { Side::Front }
    else if d < -EPSILON { Side::Back }
    else { Side::Coplanar }
}

/// A polygon (face) in CSG space.
#[derive(Debug, Clone)]
pub struct CsgPolygon {
    pub vertices: Vec<Pt3>,
    pub normal: Vec3,
}

impl CsgPolygon {
    pub fn plane(&self) -> Option<CsgPlane> {
        if self.vertices.len() < 3 { return None; }
        CsgPlane::from_normal_and_point(&self.normal, &self.vertices[0]).into()
    }

    pub fn flip(&mut self) {
        self.vertices.reverse();
        self.normal = -self.normal;
    }

    pub fn flipped(&self) -> Self {
        let mut p = self.clone();
        p.flip();
        p
    }
}

// ─── BSP Tree ──────────────────────────────────────────────────

/// BSP tree node for CSG operations.
pub struct BspNode {
    plane: Option<CsgPlane>,
    front: Option<Box<BspNode>>,
    back: Option<Box<BspNode>>,
    polygons: Vec<CsgPolygon>,
}

impl BspNode {
    pub fn new(polygons: Vec<CsgPolygon>) -> Self {
        let mut node = BspNode {
            plane: None,
            front: None,
            back: None,
            polygons: Vec::new(),
        };
        if !polygons.is_empty() {
            node.build(polygons);
        }
        node
    }

    fn build(&mut self, polygons: Vec<CsgPolygon>) {
        if polygons.is_empty() { return; }

        if self.plane.is_none() {
            // Use first polygon's plane as splitting plane
            self.plane = polygons[0].plane();
        }
        let plane = match &self.plane {
            Some(p) => p.clone(),
            None => return,
        };

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        for poly in polygons {
            split_polygon(&plane, &poly, &mut coplanar_front, &mut coplanar_back,
                         &mut front_polys, &mut back_polys);
        }
        self.polygons.extend(coplanar_front);
        self.polygons.extend(coplanar_back);

        if !front_polys.is_empty() {
            if self.front.is_none() {
                self.front = Some(Box::new(BspNode { plane: None, front: None, back: None, polygons: Vec::new() }));
            }
            self.front.as_mut().unwrap().build(front_polys);
        }

        if !back_polys.is_empty() {
            if self.back.is_none() {
                self.back = Some(Box::new(BspNode { plane: None, front: None, back: None, polygons: Vec::new() }));
            }
            self.back.as_mut().unwrap().build(back_polys);
        }
    }

    /// Invert all polygons and the BSP tree structure (swap front/back).
    pub fn invert(&mut self) {
        for poly in &mut self.polygons {
            poly.flip();
        }
        if let Some(ref mut plane) = self.plane {
            plane.normal = -plane.normal;
            plane.w = -plane.w;
        }
        if let Some(ref mut front) = self.front {
            front.invert();
        }
        if let Some(ref mut back) = self.back {
            back.invert();
        }
        std::mem::swap(&mut self.front, &mut self.back);
    }

    /// Clip polygons to this BSP tree, removing parts inside the solid.
    pub fn clip_polygons(&self, polygons: &[CsgPolygon]) -> Vec<CsgPolygon> {
        let plane = match &self.plane {
            Some(p) => p,
            None => return polygons.to_vec(),
        };

        let mut front_polys = Vec::new();
        let mut back_polys = Vec::new();
        let mut coplanar_front = Vec::new();
        let mut coplanar_back = Vec::new();

        for poly in polygons {
            split_polygon(plane, poly,
                         &mut coplanar_front, &mut coplanar_back,
                         &mut front_polys, &mut back_polys);
        }
        // Coplanar faces treated as front (kept during clipping)
        front_polys.extend(coplanar_front);
        back_polys.extend(coplanar_back);

        let front_result = if let Some(ref front) = self.front {
            front.clip_polygons(&front_polys)
        } else {
            front_polys
        };

        let back_result = if let Some(ref back) = self.back {
            back.clip_polygons(&back_polys)
        } else {
            Vec::new() // Remove back polygons (inside the solid)
        };

        let mut result = front_result;
        result.extend(back_result);
        result
    }

    /// Remove all polygons in this tree that are inside the other BSP tree.
    pub fn clip_to(&mut self, other: &BspNode) {
        self.polygons = other.clip_polygons(&self.polygons);
        if let Some(ref mut front) = self.front {
            front.clip_to(other);
        }
        if let Some(ref mut back) = self.back {
            back.clip_to(other);
        }
    }

    /// Collect all polygons from the tree.
    pub fn all_polygons(&self) -> Vec<CsgPolygon> {
        let mut result = self.polygons.clone();
        if let Some(ref front) = self.front {
            result.extend(front.all_polygons());
        }
        if let Some(ref back) = self.back {
            result.extend(back.all_polygons());
        }
        result
    }
}

// ─── Polygon Splitting ─────────────────────────────────────────

/// Split a polygon by a plane into coplanar-front, coplanar-back, front, back.
fn split_polygon(
    plane: &CsgPlane,
    polygon: &CsgPolygon,
    coplanar_front: &mut Vec<CsgPolygon>,
    coplanar_back: &mut Vec<CsgPolygon>,
    front: &mut Vec<CsgPolygon>,
    back: &mut Vec<CsgPolygon>,
) {
    let mut sides: Vec<Side> = polygon.vertices.iter()
        .map(|v| classify_point(plane, v))
        .collect();

    // Determine overall polygon classification
    let has_front = sides.iter().any(|s| *s == Side::Front);
    let has_back = sides.iter().any(|s| *s == Side::Back);

    if !has_front && !has_back {
        // All coplanar
        if polygon.normal.dot(&plane.normal) > 0.0 {
            coplanar_front.push(polygon.clone());
        } else {
            coplanar_back.push(polygon.clone());
        }
    } else if has_front && !has_back {
        front.push(polygon.clone());
    } else if !has_front && has_back {
        back.push(polygon.clone());
    } else {
        // Spanning — split the polygon
        let mut f_verts = Vec::new();
        let mut b_verts = Vec::new();
        let n = polygon.vertices.len();

        for i in 0..n {
            let j = (i + 1) % n;
            let vi = &polygon.vertices[i];
            let vj = &polygon.vertices[j];
            let si = sides[i];
            let sj = sides[j];

            if si != Side::Back {
                f_verts.push(*vi);
            }
            if si != Side::Front {
                b_verts.push(*vi);
            }

            if (si == Side::Front && sj == Side::Back) || (si == Side::Back && sj == Side::Front) {
                // Compute intersection point
                let di = plane.signed_distance(vi);
                let dj = plane.signed_distance(vj);
                let t = di / (di - dj);
                let p = Pt3::new(
                    vi.x + (vj.x - vi.x) * t,
                    vi.y + (vj.y - vi.y) * t,
                    vi.z + (vj.z - vi.z) * t,
                );
                f_verts.push(p);
                b_verts.push(p);
            }
        }

        if f_verts.len() >= 3 {
            front.push(CsgPolygon { vertices: f_verts, normal: polygon.normal });
        }
        if b_verts.len() >= 3 {
            back.push(CsgPolygon { vertices: b_verts, normal: polygon.normal });
        }
    }
}

// ─── Boolean Operations ────────────────────────────────────────

fn brep_to_polygons(brep: &BRep) -> KernelResult<Vec<CsgPolygon>> {
    let face_polys = extract_face_polygons(brep)?;
    Ok(face_polys.into_iter().map(|(verts, normal)| {
        CsgPolygon { vertices: verts, normal }
    }).collect())
}

/// Resolve T-junctions: if a vertex from one polygon lies on an edge of another
/// polygon (within tolerance), split that edge by inserting the vertex.
fn resolve_t_junctions(polygons: &mut Vec<CsgPolygon>) {
    let tol = 1e-6;
    let tol2 = tol * tol;

    // Spatial hash: quantize to grid cells for fast lookup
    let cell_size = tol * 100.0;
    let quantize = |v: f64| -> i64 { (v / cell_size).floor() as i64 };

    // Collect all unique vertices from all polygons
    let mut all_verts: Vec<Pt3> = Vec::new();
    for poly in polygons.iter() {
        for v in &poly.vertices {
            let dominated = all_verts.iter().any(|e| {
                let dx = e.x - v.x;
                let dy = e.y - v.y;
                let dz = e.z - v.z;
                dx * dx + dy * dy + dz * dz < tol2
            });
            if !dominated {
                all_verts.push(*v);
            }
        }
    }

    // Build spatial hash of vertices
    use std::collections::HashMap;
    let mut vert_grid: HashMap<(i64, i64, i64), Vec<usize>> = HashMap::new();
    for (i, v) in all_verts.iter().enumerate() {
        let key = (quantize(v.x), quantize(v.y), quantize(v.z));
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    vert_grid
                        .entry((key.0 + dx, key.1 + dy, key.2 + dz))
                        .or_default()
                        .push(i);
                }
            }
        }
    }

    // For each polygon, check each edge against nearby vertices
    for poly_idx in 0..polygons.len() {
        let n = polygons[poly_idx].vertices.len();
        let mut new_verts: Vec<Pt3> = Vec::new();

        for i in 0..n {
            let a = polygons[poly_idx].vertices[i];
            let b = polygons[poly_idx].vertices[(i + 1) % n];
            new_verts.push(a);

            let edge = Vec3::new(b.x - a.x, b.y - a.y, b.z - a.z);
            let edge_len2 = edge.x * edge.x + edge.y * edge.y + edge.z * edge.z;
            if edge_len2 < tol2 {
                continue;
            }

            // Find candidate vertices near this edge
            let mid_key = (
                quantize((a.x + b.x) * 0.5),
                quantize((a.y + b.y) * 0.5),
                quantize((a.z + b.z) * 0.5),
            );

            let mut insertions: Vec<(f64, Pt3)> = Vec::new();

            if let Some(candidates) = vert_grid.get(&mid_key) {
                for &vi in candidates {
                    let v = all_verts[vi];
                    // Skip if v is an endpoint
                    let da = (v.x - a.x) * (v.x - a.x) + (v.y - a.y) * (v.y - a.y) + (v.z - a.z) * (v.z - a.z);
                    let db = (v.x - b.x) * (v.x - b.x) + (v.y - b.y) * (v.y - b.y) + (v.z - b.z) * (v.z - b.z);
                    if da < tol2 || db < tol2 {
                        continue;
                    }

                    // Project v onto edge
                    let av = Vec3::new(v.x - a.x, v.y - a.y, v.z - a.z);
                    let t = (av.x * edge.x + av.y * edge.y + av.z * edge.z) / edge_len2;
                    if t <= tol || t >= 1.0 - tol {
                        continue;
                    }

                    // Distance from v to closest point on edge
                    let proj = Pt3::new(
                        a.x + edge.x * t,
                        a.y + edge.y * t,
                        a.z + edge.z * t,
                    );
                    let dist2 = (v.x - proj.x) * (v.x - proj.x)
                        + (v.y - proj.y) * (v.y - proj.y)
                        + (v.z - proj.z) * (v.z - proj.z);
                    if dist2 < tol2 {
                        insertions.push((t, v));
                    }
                }
            }

            // Sort by t parameter and insert
            insertions.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            for (_, pt) in insertions {
                new_verts.push(pt);
            }
        }

        if new_verts.len() > n {
            polygons[poly_idx].vertices = new_verts;
        }
    }
}

fn polygons_to_brep(polygons: &[CsgPolygon]) -> KernelResult<BRep> {
    let mut polys: Vec<CsgPolygon> = polygons.iter()
        .filter(|p| p.vertices.len() >= 3)
        .cloned()
        .collect();
    resolve_t_junctions(&mut polys);
    let faces: Vec<(Vec<Pt3>, Vec3)> = polys.iter()
        .map(|p| (p.vertices.clone(), p.normal))
        .collect();
    rebuild_brep_from_faces(&faces)
}

/// Boolean Union: A ∪ B
pub fn csg_union(a: &BRep, b: &BRep) -> KernelResult<BRep> {
    let a_polys = brep_to_polygons(a)?;
    let b_polys = brep_to_polygons(b)?;

    let mut bsp_a = BspNode::new(a_polys);
    let mut bsp_b = BspNode::new(b_polys);

    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    let mut result = bsp_a.all_polygons();
    result.extend(bsp_b.all_polygons());
    polygons_to_brep(&result)
}

/// Boolean Subtract: A - B
pub fn csg_subtract(a: &BRep, b: &BRep) -> KernelResult<BRep> {
    let a_polys = brep_to_polygons(a)?;
    let b_polys = brep_to_polygons(b)?;

    let mut bsp_a = BspNode::new(a_polys);
    let mut bsp_b = BspNode::new(b_polys);

    bsp_a.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();

    let mut result = bsp_a.all_polygons();
    result.extend(bsp_b.all_polygons());

    // Invert back
    for p in &mut result {
        p.flip();
    }

    polygons_to_brep(&result)
}

/// Boolean Intersect: A ∩ B
pub fn csg_intersect(a: &BRep, b: &BRep) -> KernelResult<BRep> {
    let a_polys = brep_to_polygons(a)?;
    let b_polys = brep_to_polygons(b)?;

    let mut bsp_a = BspNode::new(a_polys);
    let mut bsp_b = BspNode::new(b_polys);

    bsp_a.invert();
    bsp_b.clip_to(&bsp_a);
    bsp_b.invert();
    bsp_a.clip_to(&bsp_b);
    bsp_b.clip_to(&bsp_a);

    let mut result = bsp_a.all_polygons();
    result.extend(bsp_b.all_polygons());

    for p in &mut result {
        p.flip();
    }

    polygons_to_brep(&result)
}

// ─── Tests ─────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::builders::build_box_brep;

    #[test]
    fn classify_point_front_back_coplanar() {
        let plane = CsgPlane { normal: Vec3::new(0.0, 0.0, 1.0), w: 5.0 };
        assert_eq!(classify_point(&plane, &Pt3::new(0.0, 0.0, 10.0)), Side::Front);
        assert_eq!(classify_point(&plane, &Pt3::new(0.0, 0.0, 0.0)), Side::Back);
        assert_eq!(classify_point(&plane, &Pt3::new(0.0, 0.0, 5.0)), Side::Coplanar);
    }

    #[test]
    fn split_spanning_polygon() {
        let plane = CsgPlane { normal: Vec3::new(0.0, 0.0, 1.0), w: 2.5 };
        let poly = CsgPolygon {
            vertices: vec![
                Pt3::new(0.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 0.0),
                Pt3::new(5.0, 0.0, 5.0),
                Pt3::new(0.0, 0.0, 5.0),
            ],
            normal: Vec3::new(0.0, -1.0, 0.0),
        };

        let mut cf = Vec::new();
        let mut cb = Vec::new();
        let mut f = Vec::new();
        let mut b = Vec::new();
        split_polygon(&plane, &poly, &mut cf, &mut cb, &mut f, &mut b);

        assert_eq!(f.len(), 1, "Should have 1 front polygon");
        assert_eq!(b.len(), 1, "Should have 1 back polygon");
        assert!(f[0].vertices.len() >= 3);
        assert!(b[0].vertices.len() >= 3);
    }

    #[test]
    fn union_two_overlapping_boxes() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b = build_box_brep(10.0, 10.0, 10.0).unwrap(); // same position = identical
        let result = csg_union(&a, &b).unwrap();
        assert!(result.faces.len() > 0, "Union should produce faces");
    }

    #[test]
    fn union_two_disjoint_boxes() {
        let a = build_box_brep(5.0, 5.0, 5.0).unwrap();
        // b is offset — need to transform it. Use rebuild_brep_from_faces with offset.
        let b_polys = extract_face_polygons(&build_box_brep(5.0, 5.0, 5.0).unwrap()).unwrap();
        let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys.into_iter().map(|(pts, n)| {
            (pts.into_iter().map(|p| Pt3::new(p.x + 20.0, p.y, p.z)).collect(), n)
        }).collect();
        let b = rebuild_brep_from_faces(&b_offset).unwrap();

        let result = csg_union(&a, &b).unwrap();
        // Disjoint union should have faces from both boxes
        assert!(result.faces.len() >= 12, "Disjoint union should have >= 12 faces, got {}", result.faces.len());
    }

    #[test]
    fn subtract_box_from_box() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b = build_box_brep(5.0, 5.0, 20.0).unwrap(); // tall narrow box through center
        let result = csg_subtract(&a, &b).unwrap();
        assert!(result.faces.len() > 6, "Subtract should produce more faces than original");
    }

    #[test]
    fn intersect_overlapping_boxes() {
        let a = build_box_brep(10.0, 10.0, 10.0).unwrap();
        let b_polys = extract_face_polygons(&build_box_brep(10.0, 10.0, 10.0).unwrap()).unwrap();
        let b_offset: Vec<(Vec<Pt3>, Vec3)> = b_polys.into_iter().map(|(pts, n)| {
            (pts.into_iter().map(|p| Pt3::new(p.x + 5.0, p.y + 5.0, p.z)).collect(), n)
        }).collect();
        let b = rebuild_brep_from_faces(&b_offset).unwrap();

        let result = csg_intersect(&a, &b).unwrap();
        // Intersection of two offset 10×10×10 boxes = 5×5×10 region
        assert!(result.faces.len() >= 6, "Intersection should produce faces");
    }
}
