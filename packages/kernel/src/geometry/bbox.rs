use super::{Pt2, Pt3};

/// 2D axis-aligned bounding box
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox2 {
    pub min: Pt2,
    pub max: Pt2,
}

/// 3D axis-aligned bounding box
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct BoundingBox3 {
    pub min: Pt3,
    pub max: Pt3,
}

impl BoundingBox3 {
    pub fn new(min: Pt3, max: Pt3) -> Self {
        Self { min, max }
    }

    pub fn from_point(p: Pt3) -> Self {
        Self { min: p, max: p }
    }

    /// Expand this box to include a point
    pub fn include_point(&mut self, p: &Pt3) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Union of two bounding boxes
    pub fn union(&self, other: &BoundingBox3) -> BoundingBox3 {
        BoundingBox3 {
            min: Pt3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Pt3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Check if a point is inside the bounding box
    pub fn contains(&self, p: &Pt3) -> bool {
        p.x >= self.min.x
            && p.x <= self.max.x
            && p.y >= self.min.y
            && p.y <= self.max.y
            && p.z >= self.min.z
            && p.z <= self.max.z
    }

    pub fn center(&self) -> Pt3 {
        nalgebra::center(&self.min, &self.max)
    }

    pub fn diagonal(&self) -> super::Vec3 {
        self.max - self.min
    }
}

impl BoundingBox2 {
    pub fn new(min: Pt2, max: Pt2) -> Self {
        Self { min, max }
    }

    pub fn from_point(p: Pt2) -> Self {
        Self { min: p, max: p }
    }

    pub fn include_point(&mut self, p: &Pt2) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
    }

    pub fn contains(&self, p: &Pt2) -> bool {
        p.x >= self.min.x && p.x <= self.max.x && p.y >= self.min.y && p.y <= self.max.y
    }

    pub fn center(&self) -> Pt2 {
        nalgebra::center(&self.min, &self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbox3_include_point() {
        let mut bb = BoundingBox3::from_point(Pt3::origin());
        bb.include_point(&Pt3::new(1.0, 2.0, 3.0));
        bb.include_point(&Pt3::new(-1.0, -2.0, -3.0));
        assert_eq!(bb.min, Pt3::new(-1.0, -2.0, -3.0));
        assert_eq!(bb.max, Pt3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn bbox3_union() {
        let a = BoundingBox3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(1.0, 1.0, 1.0));
        let b = BoundingBox3::new(Pt3::new(-1.0, -1.0, -1.0), Pt3::new(0.5, 0.5, 0.5));
        let u = a.union(&b);
        assert_eq!(u.min, Pt3::new(-1.0, -1.0, -1.0));
        assert_eq!(u.max, Pt3::new(1.0, 1.0, 1.0));
    }

    #[test]
    fn bbox3_contains() {
        let bb = BoundingBox3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(1.0, 1.0, 1.0));
        assert!(bb.contains(&Pt3::new(0.5, 0.5, 0.5)));
        assert!(!bb.contains(&Pt3::new(2.0, 0.5, 0.5)));
    }

    #[test]
    fn bbox3_center() {
        let bb = BoundingBox3::new(Pt3::new(0.0, 0.0, 0.0), Pt3::new(2.0, 4.0, 6.0));
        let c = bb.center();
        assert_eq!(c, Pt3::new(1.0, 2.0, 3.0));
    }
}
