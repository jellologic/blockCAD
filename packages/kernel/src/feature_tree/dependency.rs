use std::collections::{HashMap, HashSet};

/// A directed acyclic graph tracking dependencies between features.
/// Used to determine which features need re-evaluation when a parameter changes.
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// feature_index -> set of feature indices it depends on
    edges: HashMap<usize, HashSet<usize>>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }

    /// Record that `dependent` depends on `dependency`.
    pub fn add_dependency(&mut self, dependent: usize, dependency: usize) {
        self.edges.entry(dependent).or_default().insert(dependency);
    }

    /// Get all features that a given feature directly depends on.
    pub fn dependencies_of(&self, index: usize) -> impl Iterator<Item = &usize> {
        self.edges
            .get(&index)
            .into_iter()
            .flat_map(|s| s.iter())
    }

    /// Get all features that transitively depend on the given feature (downstream).
    /// These are the features that need re-evaluation when `index` changes.
    pub fn downstream_of(&self, index: usize) -> HashSet<usize> {
        let mut result = HashSet::new();
        for (&feature, deps) in &self.edges {
            if deps.contains(&index) {
                result.insert(feature);
                // Recursively find downstream
                result.extend(self.downstream_of(feature));
            }
        }
        result
    }

    /// Check if adding a dependency would create a cycle.
    pub fn would_create_cycle(&self, dependent: usize, dependency: usize) -> bool {
        if dependent == dependency {
            return true;
        }
        self.downstream_of(dependent).contains(&dependency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_dependency() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(1, 0); // feature 1 depends on feature 0
        graph.add_dependency(2, 1); // feature 2 depends on feature 1

        let deps: HashSet<_> = graph.dependencies_of(2).copied().collect();
        assert!(deps.contains(&1));
        assert!(!deps.contains(&0)); // not a direct dependency

        let downstream = graph.downstream_of(0);
        assert!(downstream.contains(&1));
        assert!(downstream.contains(&2)); // transitive
    }

    #[test]
    fn cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency(1, 0);
        graph.add_dependency(2, 1);
        assert!(graph.would_create_cycle(0, 2));
        assert!(!graph.would_create_cycle(3, 0));
    }
}
