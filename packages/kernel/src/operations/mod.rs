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

pub use traits::Operation;
