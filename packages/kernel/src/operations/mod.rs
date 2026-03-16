// Client operations (always available)
pub mod traits;
pub mod extrude;
pub mod cut_extrude;
pub mod revolve;
pub mod fillet;
pub mod chamfer;
pub mod pattern;
pub mod shell;

// Server-only operations (computationally expensive)
#[cfg(feature = "server")]
pub mod boolean;
#[cfg(feature = "server")]
pub mod sweep;
#[cfg(feature = "server")]
pub mod loft;
#[cfg(feature = "server")]
pub mod draft;

pub use traits::Operation;
