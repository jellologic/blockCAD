pub mod ear_clip;
pub mod edge_tessellator;
pub mod face_tessellator;
pub mod mass_properties;
pub mod mesh;
pub mod params;

pub use face_tessellator::tessellate_brep;
pub use mass_properties::{compute_mass_properties, compute_mass_properties_with_density, MassProperties};
pub use mesh::TriMesh;
pub use params::TessellationParams;
