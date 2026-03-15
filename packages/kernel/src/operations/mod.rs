// Client operations (always available)
pub mod traits;
pub mod extrude;
pub mod revolve;
pub mod fillet;
pub mod chamfer;

// Server-only operations (computationally expensive)
#[cfg(feature = "server")]
pub mod boolean;
#[cfg(feature = "server")]
pub mod sweep;
#[cfg(feature = "server")]
pub mod loft;
#[cfg(feature = "server")]
pub mod shell;
#[cfg(feature = "server")]
pub mod draft;
#[cfg(feature = "server")]
pub mod pattern;

pub use traits::Operation;
