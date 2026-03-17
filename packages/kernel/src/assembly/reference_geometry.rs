//! Assembly-level reference geometry — planes, axes, and coordinate systems
//! that exist at the assembly level and can be used as mate targets.

use crate::geometry::Vec3;

/// Assembly-level reference geometry.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AssemblyRefGeometry {
    /// A reference plane defined by a point and normal.
    Plane {
        id: String,
        name: String,
        origin: [f64; 3],
        normal: [f64; 3],
    },
    /// A reference axis defined by a point and direction.
    Axis {
        id: String,
        name: String,
        origin: [f64; 3],
        direction: [f64; 3],
    },
    /// A coordinate system (origin + 3 axes).
    CoordinateSystem {
        id: String,
        name: String,
        origin: [f64; 3],
        x_axis: [f64; 3],
        y_axis: [f64; 3],
        z_axis: [f64; 3],
    },
}

impl AssemblyRefGeometry {
    pub fn id(&self) -> &str {
        match self {
            Self::Plane { id, .. } => id,
            Self::Axis { id, .. } => id,
            Self::CoordinateSystem { id, .. } => id,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Plane { name, .. } => name,
            Self::Axis { name, .. } => name,
            Self::CoordinateSystem { name, .. } => name,
        }
    }

    pub fn origin(&self) -> [f64; 3] {
        match self {
            Self::Plane { origin, .. } => *origin,
            Self::Axis { origin, .. } => *origin,
            Self::CoordinateSystem { origin, .. } => *origin,
        }
    }

    /// Get the primary direction/normal of this reference geometry.
    pub fn primary_direction(&self) -> [f64; 3] {
        match self {
            Self::Plane { normal, .. } => *normal,
            Self::Axis { direction, .. } => *direction,
            Self::CoordinateSystem { z_axis, .. } => *z_axis,
        }
    }

    /// Create a default XY plane at origin.
    pub fn xy_plane() -> Self {
        Self::Plane {
            id: "ref-xy".into(),
            name: "XY Plane".into(),
            origin: [0.0, 0.0, 0.0],
            normal: [0.0, 0.0, 1.0],
        }
    }

    /// Create a default XZ plane at origin.
    pub fn xz_plane() -> Self {
        Self::Plane {
            id: "ref-xz".into(),
            name: "XZ Plane".into(),
            origin: [0.0, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
        }
    }

    /// Create a default YZ plane at origin.
    pub fn yz_plane() -> Self {
        Self::Plane {
            id: "ref-yz".into(),
            name: "YZ Plane".into(),
            origin: [0.0, 0.0, 0.0],
            normal: [1.0, 0.0, 0.0],
        }
    }
}

use super::Assembly;

impl Assembly {
    /// Add reference geometry to the assembly. Returns the ID.
    pub fn add_reference_geometry(&mut self, geom: AssemblyRefGeometry) -> String {
        let id = geom.id().to_string();
        self.reference_geometry.push(geom);
        id
    }

    /// Find reference geometry by ID.
    pub fn find_reference_geometry(&self, id: &str) -> Option<&AssemblyRefGeometry> {
        self.reference_geometry.iter().find(|g| g.id() == id)
    }

    /// List all reference geometry.
    pub fn list_reference_geometry(&self) -> &[AssemblyRefGeometry] {
        &self.reference_geometry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::Assembly;

    #[test]
    fn add_and_find_reference_plane() {
        let mut asm = Assembly::new();
        asm.add_reference_geometry(AssemblyRefGeometry::xy_plane());
        let found = asm.find_reference_geometry("ref-xy");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "XY Plane");
    }

    #[test]
    fn add_reference_axis() {
        let mut asm = Assembly::new();
        let axis = AssemblyRefGeometry::Axis {
            id: "axis-1".into(),
            name: "Center Axis".into(),
            origin: [0.0, 0.0, 0.0],
            direction: [0.0, 0.0, 1.0],
        };
        asm.add_reference_geometry(axis);
        let geom = asm.find_reference_geometry("axis-1").unwrap();
        assert_eq!(geom.primary_direction(), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn add_coordinate_system() {
        let mut asm = Assembly::new();
        let cs = AssemblyRefGeometry::CoordinateSystem {
            id: "cs-1".into(),
            name: "Custom CS".into(),
            origin: [5.0, 5.0, 0.0],
            x_axis: [1.0, 0.0, 0.0],
            y_axis: [0.0, 1.0, 0.0],
            z_axis: [0.0, 0.0, 1.0],
        };
        asm.add_reference_geometry(cs);
        assert_eq!(asm.list_reference_geometry().len(), 1);
        assert_eq!(asm.list_reference_geometry()[0].origin(), [5.0, 5.0, 0.0]);
    }
}
