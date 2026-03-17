//! Assembly configurations — named snapshots of mate values and suppression states.
//!
//! Configurations allow switching between different design states of an assembly
//! (e.g., "Open" vs "Closed" for a hinge, or "Full" vs "Minimal" for optional parts).

use std::collections::HashMap;

/// A named configuration that can override mate values and component suppression.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssemblyConfig {
    pub name: String,
    /// Mate ID → overridden value (e.g., distance or angle).
    #[serde(default)]
    pub mate_value_overrides: HashMap<String, f64>,
    /// Component ID → suppression state override.
    #[serde(default)]
    pub suppression_overrides: HashMap<String, bool>,
}

impl AssemblyConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mate_value_overrides: HashMap::new(),
            suppression_overrides: HashMap::new(),
        }
    }
}

use super::Assembly;

impl Assembly {
    /// Add a new configuration. Returns its index.
    pub fn add_configuration(&mut self, config: AssemblyConfig) -> usize {
        let idx = self.configurations.len();
        self.configurations.push(config);
        idx
    }

    /// Activate a configuration by index, applying overrides.
    pub fn activate_configuration(&mut self, index: usize) -> bool {
        if index >= self.configurations.len() {
            return false;
        }
        self.active_configuration = Some(index);

        let config = self.configurations[index].clone();

        // Apply mate value overrides
        for mate in &mut self.mates {
            if let Some(&value) = config.mate_value_overrides.get(&mate.id) {
                match &mut mate.kind {
                    super::MateKind::Distance { value: v } => *v = value,
                    super::MateKind::Angle { value: v } => *v = value,
                    super::MateKind::Gear { ratio: v } => *v = value,
                    super::MateKind::Screw { pitch: v } => *v = value,
                    _ => {}
                }
            }
        }

        // Apply suppression overrides
        for comp in &mut self.components {
            if let Some(&suppressed) = config.suppression_overrides.get(&comp.id) {
                comp.suppressed = suppressed;
            }
        }

        true
    }

    /// List all configuration names.
    pub fn list_configurations(&self) -> Vec<&str> {
        self.configurations.iter().map(|c| c.name.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, Mate, MateKind, GeometryRef, Part};
    use crate::feature_tree::FeatureTree;

    fn dummy_assembly() -> Assembly {
        let mut asm = Assembly::new();
        asm.add_part(Part::new("p1", "Part", FeatureTree::new()));
        asm.add_component(Component::new("c1".into(), "p1".into(), "Comp1".into()));
        asm.add_component(Component::new("c2".into(), "p1".into(), "Comp2".into()));
        asm.mates.push(Mate {
            id: "m1".into(),
            kind: MateKind::Distance { value: 10.0 },
            component_a: "c1".into(),
            component_b: "c2".into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        });
        asm
    }

    #[test]
    fn add_and_list_configurations() {
        let mut asm = dummy_assembly();
        asm.add_configuration(AssemblyConfig::new("Default"));
        asm.add_configuration(AssemblyConfig::new("Expanded"));
        let names = asm.list_configurations();
        assert_eq!(names, vec!["Default", "Expanded"]);
    }

    #[test]
    fn activate_configuration_overrides_mate_value() {
        let mut asm = dummy_assembly();
        let mut config = AssemblyConfig::new("Wide");
        config.mate_value_overrides.insert("m1".into(), 25.0);
        asm.add_configuration(config);

        assert!(asm.activate_configuration(0));
        assert_eq!(asm.active_configuration, Some(0));
        match &asm.mates[0].kind {
            MateKind::Distance { value } => assert!((value - 25.0).abs() < 1e-10),
            _ => panic!("Expected Distance mate"),
        }
    }

    #[test]
    fn activate_configuration_overrides_suppression() {
        let mut asm = dummy_assembly();
        let mut config = AssemblyConfig::new("Minimal");
        config.suppression_overrides.insert("c2".into(), true);
        asm.add_configuration(config);

        assert!(!asm.components[1].suppressed);
        asm.activate_configuration(0);
        assert!(asm.components[1].suppressed);
    }
}
