//! Bill of Materials generation from assembly component instances.
//!
//! Supports flat BOM, hierarchical BOM with custom properties, and CSV export.

use std::collections::HashMap;
use super::Assembly;

/// A single entry in the Bill of Materials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BomEntry {
    pub part_id: String,
    pub part_name: String,
    pub quantity: usize,
}

/// An enhanced BOM entry with custom properties and nesting level.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdvancedBomEntry {
    pub part_id: String,
    pub part_name: String,
    pub quantity: usize,
    /// Nesting depth (0 = top-level).
    pub level: usize,
    /// Custom properties from the part (material, vendor, etc.).
    pub properties: HashMap<String, String>,
}

/// Generate a Bill of Materials from an assembly.
///
/// Groups active (non-suppressed) components by part_id,
/// counts instances, and returns sorted by part name.
pub fn generate_bom(assembly: &Assembly) -> Vec<BomEntry> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for comp in &assembly.components {
        if comp.suppressed {
            continue;
        }
        *counts.entry(comp.part_id.clone()).or_insert(0) += 1;
    }

    let mut entries: Vec<BomEntry> = counts
        .into_iter()
        .map(|(part_id, quantity)| {
            let part_name = assembly
                .find_part(&part_id)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| part_id.clone());
            BomEntry { part_id, part_name, quantity }
        })
        .collect();

    entries.sort_by(|a, b| a.part_name.cmp(&b.part_name));
    entries
}

/// Generate an advanced BOM with custom part properties.
///
/// Includes material, vendor, and other properties stored on each Part.
pub fn generate_advanced_bom(assembly: &Assembly) -> Vec<AdvancedBomEntry> {
    let mut counts: HashMap<String, usize> = HashMap::new();

    for comp in &assembly.components {
        if comp.suppressed {
            continue;
        }
        *counts.entry(comp.part_id.clone()).or_insert(0) += 1;
    }

    let mut entries: Vec<AdvancedBomEntry> = counts
        .into_iter()
        .map(|(part_id, quantity)| {
            let part = assembly.find_part(&part_id);
            let part_name = part.map(|p| p.name.clone()).unwrap_or_else(|| part_id.clone());
            let properties = part.map(|p| p.properties.clone()).unwrap_or_default();
            AdvancedBomEntry {
                part_id,
                part_name,
                quantity,
                level: 0,
                properties,
            }
        })
        .collect();

    entries.sort_by(|a, b| a.part_name.cmp(&b.part_name));
    entries
}

/// Export BOM as CSV string.
pub fn bom_to_csv(entries: &[AdvancedBomEntry]) -> String {
    let mut out = String::new();

    // Collect all unique property keys
    let mut property_keys: Vec<String> = entries
        .iter()
        .flat_map(|e| e.properties.keys().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    property_keys.sort();

    // Header
    out.push_str("Item,Part ID,Part Name,Quantity");
    for key in &property_keys {
        out.push(',');
        out.push_str(&csv_escape(key));
    }
    out.push('\n');

    // Rows
    for (i, entry) in entries.iter().enumerate() {
        let indent = "  ".repeat(entry.level);
        out.push_str(&format!("{},{},{}{},{}", i + 1,
            csv_escape(&entry.part_id),
            indent,
            csv_escape(&entry.part_name),
            entry.quantity,
        ));
        for key in &property_keys {
            out.push(',');
            if let Some(val) = entry.properties.get(key) {
                out.push_str(&csv_escape(val));
            }
        }
        out.push('\n');
    }

    out
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::FeatureTree;

    fn dummy_part(id: &str, name: &str) -> Part {
        Part::new(id, name, FeatureTree::new())
    }

    #[test]
    fn bom_counts_instances() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("bolt", "M6 Bolt"));
        for i in 0..3 {
            assembly.add_component(Component::new(
                format!("b{}", i), "bolt".into(), format!("Bolt {}", i),
            ));
        }
        let bom = generate_bom(&assembly);
        assert_eq!(bom.len(), 1);
        assert_eq!(bom[0].part_name, "M6 Bolt");
        assert_eq!(bom[0].quantity, 3);
    }

    #[test]
    fn bom_multiple_parts_sorted() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("plate", "Zeta Plate"));
        assembly.add_part(dummy_part("bolt", "Alpha Bolt"));
        assembly.add_component(Component::new("c1".into(), "plate".into(), "Plate".into()));
        assembly.add_component(Component::new("c2".into(), "bolt".into(), "Bolt 1".into()));
        assembly.add_component(Component::new("c3".into(), "bolt".into(), "Bolt 2".into()));

        let bom = generate_bom(&assembly);
        assert_eq!(bom.len(), 2);
        assert_eq!(bom[0].part_name, "Alpha Bolt");
        assert_eq!(bom[0].quantity, 2);
        assert_eq!(bom[1].part_name, "Zeta Plate");
        assert_eq!(bom[1].quantity, 1);
    }

    #[test]
    fn bom_excludes_suppressed() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("bolt", "Bolt"));
        assembly.add_component(Component::new("c1".into(), "bolt".into(), "Active".into()));
        let mut suppressed = Component::new("c2".into(), "bolt".into(), "Hidden".into());
        suppressed.suppressed = true;
        assembly.add_component(suppressed);

        let bom = generate_bom(&assembly);
        assert_eq!(bom[0].quantity, 1);
    }

    #[test]
    fn bom_empty_assembly() {
        let assembly = Assembly::new();
        let bom = generate_bom(&assembly);
        assert!(bom.is_empty());
    }

    #[test]
    fn advanced_bom_includes_properties() {
        let mut assembly = Assembly::new();
        let mut part = Part::new("bolt", "M6 Bolt", FeatureTree::new());
        part.properties.insert("material".into(), "Steel".into());
        part.properties.insert("vendor".into(), "ACME".into());
        assembly.add_part(part);
        assembly.add_component(Component::new("c1".into(), "bolt".into(), "Bolt 1".into()));
        assembly.add_component(Component::new("c2".into(), "bolt".into(), "Bolt 2".into()));

        let bom = generate_advanced_bom(&assembly);
        assert_eq!(bom.len(), 1);
        assert_eq!(bom[0].quantity, 2);
        assert_eq!(bom[0].properties.get("material").unwrap(), "Steel");
        assert_eq!(bom[0].properties.get("vendor").unwrap(), "ACME");
    }

    #[test]
    fn csv_export_has_header_and_rows() {
        let entries = vec![
            AdvancedBomEntry {
                part_id: "p1".into(),
                part_name: "Bolt".into(),
                quantity: 4,
                level: 0,
                properties: [("material".into(), "Steel".into())].into_iter().collect(),
            },
            AdvancedBomEntry {
                part_id: "p2".into(),
                part_name: "Plate".into(),
                quantity: 1,
                level: 0,
                properties: [("material".into(), "Aluminum".into())].into_iter().collect(),
            },
        ];
        let csv = bom_to_csv(&entries);
        assert!(csv.contains("Part ID"));
        assert!(csv.contains("material"));
        assert!(csv.contains("Bolt"));
        assert!(csv.contains("Steel"));
        assert!(csv.contains("Aluminum"));
    }

    #[test]
    fn csv_escape_handles_commas() {
        let escaped = csv_escape("Hello, World");
        assert_eq!(escaped, "\"Hello, World\"");
    }
}
