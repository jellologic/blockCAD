use crate::geometry::Pt2;
use super::entity::SketchEntityId;

/// A sketch block definition: a named group of sketch entities
/// that can be reused by inserting instances.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SketchBlock {
    pub id: String,
    pub name: String,
    /// Insertion (anchor) point for the block.
    pub insertion_point: Pt2,
    /// Indices of the sketch entities belonging to this block.
    pub entity_indices: Vec<SketchEntityId>,
}

/// An instance of a block placed in the sketch.
/// The instance transforms the block's entities via position, scale, and rotation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SketchBlockInstance {
    pub id: String,
    pub block_id: String,
    pub position: Pt2,
    pub scale: f64,
    /// Rotation angle in radians.
    pub rotation: f64,
}

impl SketchBlockInstance {
    /// Transform a point from block-local space into sketch space.
    pub fn transform_point(&self, pt: Pt2, block_origin: Pt2) -> Pt2 {
        let dx = pt.x - block_origin.x;
        let dy = pt.y - block_origin.y;
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        Pt2::new(
            self.position.x + self.scale * (cos * dx - sin * dy),
            self.position.y + self.scale * (sin * dx + cos * dy),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_instance_identity_transform() {
        let instance = SketchBlockInstance {
            id: "bi-0".into(),
            block_id: "b-0".into(),
            position: Pt2::new(0.0, 0.0),
            scale: 1.0,
            rotation: 0.0,
        };
        let origin = Pt2::new(0.0, 0.0);
        let pt = Pt2::new(3.0, 4.0);
        let result = instance.transform_point(pt, origin);
        assert!((result.x - 3.0).abs() < 1e-9);
        assert!((result.y - 4.0).abs() < 1e-9);
    }

    #[test]
    fn block_instance_translate() {
        let instance = SketchBlockInstance {
            id: "bi-0".into(),
            block_id: "b-0".into(),
            position: Pt2::new(10.0, 20.0),
            scale: 1.0,
            rotation: 0.0,
        };
        let origin = Pt2::new(0.0, 0.0);
        let pt = Pt2::new(1.0, 2.0);
        let result = instance.transform_point(pt, origin);
        assert!((result.x - 11.0).abs() < 1e-9);
        assert!((result.y - 22.0).abs() < 1e-9);
    }

    #[test]
    fn block_instance_scale_and_rotate() {
        let instance = SketchBlockInstance {
            id: "bi-0".into(),
            block_id: "b-0".into(),
            position: Pt2::new(0.0, 0.0),
            scale: 2.0,
            rotation: std::f64::consts::FRAC_PI_2, // 90 degrees
        };
        let origin = Pt2::new(0.0, 0.0);
        let pt = Pt2::new(1.0, 0.0);
        let result = instance.transform_point(pt, origin);
        // 90-deg rotation of (1,0) → (0,1), scaled by 2 → (0,2)
        assert!(result.x.abs() < 1e-9);
        assert!((result.y - 2.0).abs() < 1e-9);
    }
}
