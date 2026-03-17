//! STEP (ISO 10303-21) assembly export.
//!
//! Generates AP203/AP214 STEP files with assembly structure:
//! - PRODUCT + PRODUCT_DEFINITION per part
//! - NEXT_ASSEMBLY_USAGE_OCCURRENCE per component instance
//! - AXIS2_PLACEMENT_3D for component transforms

use std::fmt::Write;

use crate::error::KernelResult;
use crate::geometry::transform::from_array;

/// Options for STEP export.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StepOptions {
    /// Application protocol: "AP203" or "AP214" (default: "AP214")
    #[serde(default = "default_protocol")]
    pub protocol: String,
    /// Author name (default: "blockCAD")
    #[serde(default = "default_author")]
    pub author: String,
}

fn default_protocol() -> String { "AP214".into() }
fn default_author() -> String { "blockCAD".into() }

impl Default for StepOptions {
    fn default() -> Self {
        Self { protocol: default_protocol(), author: default_author() }
    }
}

/// Component data for STEP assembly export.
pub struct StepComponent {
    pub id: String,
    pub name: String,
    pub part_name: String,
    pub transform: [f64; 16],
}

/// Export an assembly as STEP text.
///
/// Creates PRODUCT entities for each unique part, PRODUCT_DEFINITION for each,
/// and NEXT_ASSEMBLY_USAGE_OCCURRENCE for each component instance.
pub fn export_step_assembly(
    assembly_name: &str,
    components: &[StepComponent],
    options: &StepOptions,
) -> KernelResult<String> {
    let mut out = String::new();
    let mut entity_id: usize = 1;

    // Header
    writeln!(out, "ISO-10303-21;").unwrap();
    writeln!(out, "HEADER;").unwrap();
    writeln!(out, "FILE_DESCRIPTION(('blockCAD STEP Export'),'2;1');").unwrap();
    writeln!(out, "FILE_NAME('assembly.step','{}','(\\'{}\\')','','blockCAD','','');",
        chrono_placeholder(), options.author).unwrap();
    writeln!(out, "FILE_SCHEMA(('AUTOMOTIVE_DESIGN'));").unwrap();
    writeln!(out, "ENDSEC;").unwrap();
    writeln!(out, "DATA;").unwrap();

    // Application context
    let app_ctx = entity_id;
    writeln!(out, "#{}=APPLICATION_CONTEXT('automotive design');", entity_id).unwrap();
    entity_id += 1;

    let app_proto = entity_id;
    writeln!(out, "#{}=APPLICATION_PROTOCOL_DEFINITION('international standard','automotive_design',2000,#{});",
        entity_id, app_ctx).unwrap();
    entity_id += 1;

    // Assembly root product
    let asm_product = entity_id;
    writeln!(out, "#{}=PRODUCT('{}','{}','',(#{}));",
        entity_id, assembly_name, assembly_name, entity_id + 1).unwrap();
    entity_id += 1;

    let asm_pdc = entity_id;
    writeln!(out, "#{}=PRODUCT_DEFINITION_CONTEXT('part definition',#{});",
        entity_id, app_ctx).unwrap();
    entity_id += 1;

    let asm_pdf = entity_id;
    writeln!(out, "#{}=PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE('','',#{}, .NOT_KNOWN.);",
        entity_id, asm_product).unwrap();
    entity_id += 1;

    let asm_pd = entity_id;
    writeln!(out, "#{}=PRODUCT_DEFINITION('design','',#{},#{});",
        entity_id, asm_pdf, asm_pdc).unwrap();
    entity_id += 1;

    // Track unique parts to avoid duplicates
    let mut part_products: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

    for comp in components {
        // Create part product if not already created
        let part_pd = if let Some(&existing) = part_products.get(&comp.part_name) {
            existing
        } else {
            let part_product = entity_id;
            writeln!(out, "#{}=PRODUCT('{}','{}','',(#{}));",
                entity_id, comp.part_name, comp.part_name, entity_id + 1).unwrap();
            entity_id += 1;

            let _part_pdc = entity_id;
            writeln!(out, "#{}=PRODUCT_DEFINITION_CONTEXT('part definition',#{});",
                entity_id, app_ctx).unwrap();
            entity_id += 1;

            let part_pdf = entity_id;
            writeln!(out, "#{}=PRODUCT_DEFINITION_FORMATION_WITH_SPECIFIED_SOURCE('','',#{}, .NOT_KNOWN.);",
                entity_id, part_product).unwrap();
            entity_id += 1;

            let part_pd = entity_id;
            writeln!(out, "#{}=PRODUCT_DEFINITION('design','',#{},#{});",
                entity_id, part_pdf, _part_pdc).unwrap();
            entity_id += 1;

            part_products.insert(comp.part_name.clone(), part_pd);
            part_pd
        };

        // Component transform as AXIS2_PLACEMENT_3D
        let transform = from_array(&comp.transform);

        // Extract origin, Z-direction, X-direction from the 4x4 matrix
        let origin = [transform[(0, 3)], transform[(1, 3)], transform[(2, 3)]];
        let z_dir = [transform[(0, 2)], transform[(1, 2)], transform[(2, 2)]];
        let x_dir = [transform[(0, 0)], transform[(1, 0)], transform[(2, 0)]];

        let origin_id = entity_id;
        writeln!(out, "#{}=CARTESIAN_POINT('',({:.6},{:.6},{:.6}));",
            entity_id, origin[0], origin[1], origin[2]).unwrap();
        entity_id += 1;

        let z_dir_id = entity_id;
        writeln!(out, "#{}=DIRECTION('',({:.6},{:.6},{:.6}));",
            entity_id, z_dir[0], z_dir[1], z_dir[2]).unwrap();
        entity_id += 1;

        let x_dir_id = entity_id;
        writeln!(out, "#{}=DIRECTION('',({:.6},{:.6},{:.6}));",
            entity_id, x_dir[0], x_dir[1], x_dir[2]).unwrap();
        entity_id += 1;

        let placement_id = entity_id;
        writeln!(out, "#{}=AXIS2_PLACEMENT_3D('',#{},#{},#{});",
            entity_id, origin_id, z_dir_id, x_dir_id).unwrap();
        entity_id += 1;

        // NEXT_ASSEMBLY_USAGE_OCCURRENCE
        writeln!(out, "#{}=NEXT_ASSEMBLY_USAGE_OCCURRENCE('{}','{}','',#{},#{});",
            entity_id, comp.id, comp.name, asm_pd, part_pd).unwrap();
        entity_id += 1;

        // Item-defined transform linking placement to the usage
        writeln!(out,
            "#{}=ITEM_DEFINED_TRANSFORMATION('','',#{},#{});",
            entity_id, placement_id, placement_id).unwrap();
        entity_id += 1;
    }

    writeln!(out, "ENDSEC;").unwrap();
    writeln!(out, "END-ISO-10303-21;").unwrap();

    Ok(out)
}

fn chrono_placeholder() -> String {
    "2026-03-17T00:00:00".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::transform;

    fn make_components() -> Vec<StepComponent> {
        vec![
            StepComponent {
                id: "comp-1".into(),
                name: "Plate A".into(),
                part_name: "Plate".into(),
                transform: transform::to_array(&transform::translation(0.0, 0.0, 0.0)),
            },
            StepComponent {
                id: "comp-2".into(),
                name: "Plate B".into(),
                part_name: "Plate".into(),
                transform: transform::to_array(&transform::translation(20.0, 0.0, 0.0)),
            },
            StepComponent {
                id: "comp-3".into(),
                name: "Bolt".into(),
                part_name: "Bolt".into(),
                transform: transform::to_array(&transform::translation(10.0, 5.0, 0.0)),
            },
        ]
    }

    #[test]
    fn step_export_has_header_and_footer() {
        let components = make_components();
        let step = export_step_assembly("Test Assembly", &components, &StepOptions::default()).unwrap();
        assert!(step.starts_with("ISO-10303-21;"));
        assert!(step.contains("END-ISO-10303-21;"));
        assert!(step.contains("HEADER;"));
        assert!(step.contains("DATA;"));
    }

    #[test]
    fn step_export_contains_products() {
        let components = make_components();
        let step = export_step_assembly("My Assembly", &components, &StepOptions::default()).unwrap();
        assert!(step.contains("PRODUCT('My Assembly'"));
        assert!(step.contains("PRODUCT('Plate'"));
        assert!(step.contains("PRODUCT('Bolt'"));
    }

    #[test]
    fn step_export_contains_assembly_usage() {
        let components = make_components();
        let step = export_step_assembly("Asm", &components, &StepOptions::default()).unwrap();
        // Should have 3 NAUO entries (one per component)
        let nauo_count = step.matches("NEXT_ASSEMBLY_USAGE_OCCURRENCE").count();
        assert_eq!(nauo_count, 3);
        // Should have 3 AXIS2_PLACEMENT_3D entries
        let placement_count = step.matches("AXIS2_PLACEMENT_3D").count();
        assert_eq!(placement_count, 3);
    }
}
