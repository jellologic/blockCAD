/// Parameters controlling tessellation quality.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TessellationParams {
    /// Maximum chord deviation from true surface (in model units)
    pub chord_tolerance: f64,
    /// Maximum angle between adjacent normals (in radians)
    pub angle_tolerance: f64,
    /// Minimum edge length for subdivision
    pub min_edge_length: f64,
}

impl Default for TessellationParams {
    fn default() -> Self {
        Self {
            chord_tolerance: 0.01,
            angle_tolerance: 0.5_f64.to_radians(),
            min_edge_length: 0.001,
        }
    }
}

impl TessellationParams {
    /// High quality preset
    pub fn high_quality() -> Self {
        Self {
            chord_tolerance: 0.001,
            angle_tolerance: 0.25_f64.to_radians(),
            min_edge_length: 0.0001,
        }
    }

    /// Low quality preset for fast preview
    pub fn preview() -> Self {
        Self {
            chord_tolerance: 0.1,
            angle_tolerance: 1.0_f64.to_radians(),
            min_edge_length: 0.01,
        }
    }
}
