// Client operations (always available)
pub mod traits;
pub mod extrude;
pub mod cut_extrude;
pub mod revolve;
pub mod fillet;
pub mod chamfer;
pub mod pattern;
pub mod shell;

pub mod boolean;

pub mod sweep;
pub mod loft;
pub mod draft;
pub mod datum_plane;
pub mod transform;
pub mod transform_body;
pub mod hole;
pub mod dome;
pub mod rib;

pub use traits::Operation;
