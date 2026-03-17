//! Bill of Materials generation from assembly component instances.

use std::collections::HashMap;
use super::Assembly;

/// A single entry in the Bill of Materials.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BomEntry {
    pub part_id: String,
    pub part_name: String,
    pub quantity: usize,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Part};
    use crate::feature_tree::FeatureTree;

    fn dummy_part(id: &str, name: &str) -> Part {
        Part { id: id.into(), name: name.into(), tree: FeatureTree::new(), density: 1.0 }
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
        // Sorted alphabetically: Alpha Bolt before Zeta Plate
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
        assert_eq!(bom[0].quantity, 1); // Only the active one
    }

    #[test]
    fn bom_empty_assembly() {
        let assembly = Assembly::new();
        let bom = generate_bom(&assembly);
        assert!(bom.is_empty());
    }
}
