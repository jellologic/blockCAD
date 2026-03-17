//! Assembly report generation — structured summary with BOM, interference, and mass properties.

use super::Assembly;
use super::bom::{self, BomEntry};
use super::interference::{self, Interference};
use super::mass::{self, AssemblyMassProperties};
use crate::topology::BRep;

/// Structured assembly report.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyReport {
    pub name: String,
    pub summary: ReportSummary,
    pub bom: Vec<BomEntry>,
    pub interferences: Vec<InterferenceEntry>,
    pub mass_properties: AssemblyMassProperties,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportSummary {
    pub total_parts: usize,
    pub total_components: usize,
    pub active_components: usize,
    pub total_mates: usize,
    pub active_mates: usize,
    pub has_interferences: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterferenceEntry {
    pub component_a: String,
    pub component_b: String,
    pub overlap_distance: f64,
}

/// Generate a full assembly report.
pub fn generate_report(
    assembly: &Assembly,
    name: &str,
    component_breps: &[(String, BRep)],
) -> AssemblyReport {
    let bom_entries = bom::generate_bom(assembly);
    let interferences = interference::check_interference(component_breps)
        .unwrap_or_default();
    let mass_props = mass::compute_mass_properties(component_breps);

    let active_components = assembly.components.iter().filter(|c| !c.suppressed).count();
    let active_mates = assembly.mates.iter().filter(|m| !m.suppressed).count();

    let interference_entries: Vec<InterferenceEntry> = interferences
        .iter()
        .map(|i| InterferenceEntry {
            component_a: i.component_a.clone(),
            component_b: i.component_b.clone(),
            overlap_distance: i.overlap_distance,
        })
        .collect();

    AssemblyReport {
        name: name.into(),
        summary: ReportSummary {
            total_parts: assembly.parts.len(),
            total_components: assembly.components.len(),
            active_components,
            total_mates: assembly.mates.len(),
            active_mates,
            has_interferences: !interference_entries.is_empty(),
        },
        bom: bom_entries,
        interferences: interference_entries,
        mass_properties: mass_props,
    }
}

/// Render the report as an HTML string.
pub fn report_to_html(report: &AssemblyReport) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><meta charset='utf-8'>");
    html.push_str("<title>Assembly Report</title>");
    html.push_str("<style>body{font-family:sans-serif;margin:2em}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:6px 10px;text-align:left}th{background:#f4f4f4}.warn{color:#c00}</style>");
    html.push_str("</head><body>");

    html.push_str(&format!("<h1>Assembly Report: {}</h1>", report.name));

    // Summary
    html.push_str("<h2>Summary</h2><ul>");
    html.push_str(&format!("<li>Parts: {}</li>", report.summary.total_parts));
    html.push_str(&format!("<li>Components: {} ({} active)</li>",
        report.summary.total_components, report.summary.active_components));
    html.push_str(&format!("<li>Mates: {} ({} active)</li>",
        report.summary.total_mates, report.summary.active_mates));
    if report.summary.has_interferences {
        html.push_str(&format!("<li class='warn'>Interferences: {}</li>", report.interferences.len()));
    } else {
        html.push_str("<li>No interferences detected</li>");
    }
    html.push_str("</ul>");

    // BOM
    html.push_str("<h2>Bill of Materials</h2>");
    html.push_str("<table><thead><tr><th>#</th><th>Part</th><th>Qty</th></tr></thead><tbody>");
    for (i, entry) in report.bom.iter().enumerate() {
        html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>",
            i + 1, entry.part_name, entry.quantity));
    }
    html.push_str("</tbody></table>");

    // Mass properties
    html.push_str("<h2>Mass Properties</h2><ul>");
    html.push_str(&format!("<li>Volume: {:.2} cubic units</li>", report.mass_properties.total_volume));
    html.push_str(&format!("<li>COG: ({:.2}, {:.2}, {:.2})</li>",
        report.mass_properties.center_of_gravity[0],
        report.mass_properties.center_of_gravity[1],
        report.mass_properties.center_of_gravity[2]));
    html.push_str("</ul>");

    // Interferences
    if !report.interferences.is_empty() {
        html.push_str("<h2>Interferences</h2><table><thead><tr><th>Component A</th><th>Component B</th><th>Overlap</th></tr></thead><tbody>");
        for i in &report.interferences {
            html.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{:.3}</td></tr>",
                i.component_a, i.component_b, i.overlap_distance));
        }
        html.push_str("</tbody></table>");
    }

    html.push_str("</body></html>");
    html
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::FeatureTree;
    use crate::topology::builders::build_box_brep;

    fn setup() -> (Assembly, Vec<(String, BRep)>) {
        let mut asm = Assembly::new();
        asm.add_part(Part::new("p1", "Plate", FeatureTree::new()));
        asm.add_component(Component::new("c1".into(), "p1".into(), "Plate A".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "Plate B".into()));

        let brep1 = build_box_brep(10.0, 10.0, 2.0).unwrap();
        let brep2 = build_box_brep(10.0, 10.0, 2.0).unwrap();
        let breps = vec![("c1".into(), brep1), ("c2".into(), brep2)];
        (asm, breps)
    }

    #[test]
    fn report_summary_counts() {
        let (asm, breps) = setup();
        let report = generate_report(&asm, "Test Assembly", &breps);
        assert_eq!(report.summary.total_parts, 1);
        assert_eq!(report.summary.total_components, 2);
        assert_eq!(report.summary.active_components, 2);
        assert_eq!(report.name, "Test Assembly");
    }

    #[test]
    fn report_html_contains_title() {
        let (asm, breps) = setup();
        let report = generate_report(&asm, "My Assembly", &breps);
        let html = report_to_html(&report);
        assert!(html.contains("My Assembly"));
        assert!(html.contains("<table>"));
        assert!(html.contains("Plate"));
    }
}
