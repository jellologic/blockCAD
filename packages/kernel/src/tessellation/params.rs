/// Parameters controlling tessellation quality.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TessellationParams {
    /// Maximum chord deviation from true surface (in model units)
    pub chord_tolerance: f64,
    /// Maximum angle between adjacent normals (in radians)
    pub angle_tolerance: f64,
    /// Minimum edge length for subdivision
    pub min_edge_length: f64,
    /// Skip `fix_winding()` and `validate()` post-processing for faster
    /// viewport tessellation.  When `true` the mesh is returned immediately
    /// after triangulation, which is roughly 30 % faster but may contain
    /// incorrect winding or fail strict validation checks.
    #[serde(default)]
    pub skip_validation: bool,
}

impl Default for TessellationParams {
    fn default() -> Self {
        Self {
            chord_tolerance: 0.01,
            angle_tolerance: 0.5_f64.to_radians(),
            min_edge_length: 0.001,
            skip_validation: false,
        }
    }
}

impl TessellationParams {
    /// High quality preset for detailed export
    pub fn high_quality() -> Self {
        Self {
            chord_tolerance: 0.001,
            angle_tolerance: 0.25_f64.to_radians(),
            min_edge_length: 0.0001,
            skip_validation: false,
        }
    }

    /// Viewport-optimized preset — 5x coarser than default for interactive display,
    /// skips `fix_winding()` and `validate()` for speed.
    pub fn viewport() -> Self {
        Self {
            chord_tolerance: 0.05,
            angle_tolerance: 1.0_f64.to_radians(),
            min_edge_length: 0.01,
            skip_validation: true,
        }
    }

    /// Export quality preset — identical to default tight tolerances
    pub fn export_quality() -> Self {
        Self::default()
    }

    /// Very coarse preset for drag/preview operations
    pub fn preview() -> Self {
        Self {
            chord_tolerance: 0.2,
            angle_tolerance: 2.0_f64.to_radians(),
            min_edge_length: 0.05,
            skip_validation: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_have_increasing_quality() {
        let preview = TessellationParams::preview();
        let viewport = TessellationParams::viewport();
        let default = TessellationParams::default();
        let export = TessellationParams::export_quality();
        let high = TessellationParams::high_quality();

        // Coarser presets have larger tolerances (fewer triangles)
        assert!(preview.chord_tolerance > viewport.chord_tolerance);
        assert!(viewport.chord_tolerance > default.chord_tolerance);
        assert!(default.chord_tolerance > high.chord_tolerance);

        // export_quality equals default
        assert_eq!(export.chord_tolerance, default.chord_tolerance);
        assert_eq!(export.angle_tolerance, default.angle_tolerance);
        assert_eq!(export.min_edge_length, default.min_edge_length);
    }

    #[test]
    fn preview_is_coarsest() {
        let preview = TessellationParams::preview();
        assert!(preview.chord_tolerance >= 0.2);
        assert!(preview.min_edge_length >= 0.05);
    }

    #[test]
    fn high_quality_is_finest() {
        let hq = TessellationParams::high_quality();
        assert!(hq.chord_tolerance <= 0.001);
        assert!(hq.min_edge_length <= 0.0001);
    }

    #[test]
    fn viewport_skips_validation() {
        let viewport = TessellationParams::viewport();
        assert!(viewport.skip_validation);
    }

    #[test]
    fn default_does_not_skip_validation() {
        let default = TessellationParams::default();
        assert!(!default.skip_validation);
    }
}
