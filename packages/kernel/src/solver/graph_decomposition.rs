//! Graph decomposition for assembly solving.
//!
//! Decomposes an assembly's mate graph into independent connected clusters,
//! allowing each cluster to be solved separately for better performance.

use std::collections::{HashMap, HashSet, VecDeque};

use crate::assembly::Assembly;

/// A cluster of components connected by mates.
#[derive(Debug)]
pub struct ComponentCluster {
    /// Component IDs in this cluster.
    pub component_ids: Vec<String>,
    /// Indices into `assembly.mates` for mates within this cluster.
    pub mate_indices: Vec<usize>,
}

/// Decompose an assembly into independent connected clusters.
///
/// Components connected (directly or transitively) through mates form a cluster.
/// Components with no mates are singletons.
pub fn decompose(assembly: &Assembly) -> Vec<ComponentCluster> {
    let active_components: Vec<&str> = assembly
        .components
        .iter()
        .filter(|c| !c.suppressed)
        .map(|c| c.id.as_str())
        .collect();

    if active_components.is_empty() {
        return Vec::new();
    }

    // Build adjacency list from active (non-suppressed) mates
    let mut adjacency: HashMap<&str, HashSet<&str>> = HashMap::new();
    let mut mate_map: HashMap<(&str, &str), Vec<usize>> = HashMap::new();

    for (i, mate) in assembly.mates.iter().enumerate() {
        if mate.suppressed {
            continue;
        }
        let a = mate.component_a.as_str();
        let b = mate.component_b.as_str();
        adjacency.entry(a).or_default().insert(b);
        adjacency.entry(b).or_default().insert(a);
        mate_map.entry((a, b)).or_default().push(i);
        mate_map.entry((b, a)).or_default().push(i);
    }

    // BFS to find connected components
    let mut visited: HashSet<&str> = HashSet::new();
    let mut clusters: Vec<ComponentCluster> = Vec::new();

    for &comp_id in &active_components {
        if visited.contains(comp_id) {
            continue;
        }

        let mut cluster_ids: Vec<String> = Vec::new();
        let mut cluster_mates: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();
        queue.push_back(comp_id);
        visited.insert(comp_id);

        while let Some(current) = queue.pop_front() {
            cluster_ids.push(current.to_string());

            if let Some(neighbors) = adjacency.get(current) {
                for &neighbor in neighbors {
                    // Collect mate indices
                    if let Some(indices) = mate_map.get(&(current, neighbor)) {
                        for &idx in indices {
                            cluster_mates.insert(idx);
                        }
                    }
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor);
                        queue.push_back(neighbor);
                    }
                }
            }
        }

        let mut mate_indices: Vec<usize> = cluster_mates.into_iter().collect();
        mate_indices.sort();

        clusters.push(ComponentCluster {
            component_ids: cluster_ids,
            mate_indices,
        });
    }

    clusters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assembly::{Assembly, Component, GeometryRef, Mate, MateKind, Part};
    use crate::feature_tree::FeatureTree;

    fn dummy_part(id: &str) -> Part {
        Part::new(id, id, FeatureTree::new())
    }

    fn dummy_mate(id: &str, a: &str, b: &str) -> Mate {
        Mate {
            id: id.into(),
            kind: MateKind::Coincident,
            component_a: a.into(),
            component_b: b.into(),
            geometry_ref_a: GeometryRef::Face(0),
            geometry_ref_b: GeometryRef::Face(0),
            suppressed: false,
        }
    }

    #[test]
    fn empty_assembly_no_clusters() {
        let assembly = Assembly::new();
        let clusters = decompose(&assembly);
        assert!(clusters.is_empty());
    }

    #[test]
    fn single_component_singleton_cluster() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("p1"));
        assembly.add_component(Component::new("c1".into(), "p1".into(), "C1".into()));
        let clusters = decompose(&assembly);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].component_ids.len(), 1);
        assert!(clusters[0].mate_indices.is_empty());
    }

    #[test]
    fn two_mated_components_one_cluster() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("p1"));
        assembly.add_component(Component::new("c1".into(), "p1".into(), "C1".into()));
        assembly.add_component(Component::new("c2".into(), "p1".into(), "C2".into()));
        assembly.mates.push(dummy_mate("m1", "c1", "c2"));
        let clusters = decompose(&assembly);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].component_ids.len(), 2);
        assert_eq!(clusters[0].mate_indices.len(), 1);
    }

    #[test]
    fn disconnected_components_two_clusters() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("p1"));
        assembly.add_component(Component::new("c1".into(), "p1".into(), "C1".into()));
        assembly.add_component(Component::new("c2".into(), "p1".into(), "C2".into()));
        assembly.add_component(Component::new("c3".into(), "p1".into(), "C3".into()));
        assembly.add_component(Component::new("c4".into(), "p1".into(), "C4".into()));
        // c1-c2 connected, c3-c4 connected, no link between groups
        assembly.mates.push(dummy_mate("m1", "c1", "c2"));
        assembly.mates.push(dummy_mate("m2", "c3", "c4"));
        let clusters = decompose(&assembly);
        assert_eq!(clusters.len(), 2);
    }

    #[test]
    fn chain_of_mates_single_cluster() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("p1"));
        for i in 0..5 {
            assembly.add_component(Component::new(format!("c{}", i), "p1".into(), format!("C{}", i)));
        }
        // Chain: c0-c1, c1-c2, c2-c3, c3-c4
        for i in 0..4 {
            assembly.mates.push(dummy_mate(&format!("m{}", i), &format!("c{}", i), &format!("c{}", i + 1)));
        }
        let clusters = decompose(&assembly);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].component_ids.len(), 5);
        assert_eq!(clusters[0].mate_indices.len(), 4);
    }

    #[test]
    fn suppressed_mate_excluded() {
        let mut assembly = Assembly::new();
        assembly.add_part(dummy_part("p1"));
        assembly.add_component(Component::new("c1".into(), "p1".into(), "C1".into()));
        assembly.add_component(Component::new("c2".into(), "p1".into(), "C2".into()));
        let mut mate = dummy_mate("m1", "c1", "c2");
        mate.suppressed = true;
        assembly.mates.push(mate);
        let clusters = decompose(&assembly);
        // With suppressed mate, c1 and c2 are disconnected → 2 clusters
        assert_eq!(clusters.len(), 2);
    }
}
