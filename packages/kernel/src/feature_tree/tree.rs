use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::operations::extrude::ExtrudeProfile;
use crate::sketch::Sketch;
use crate::topology::brep::BRepFingerprint;
use crate::topology::BRep;

use super::feature::{Feature, FeatureState};

/// The parametric feature tree with rollback cursor and evaluation cache.
///
/// Features are stored in order. The cursor indicates which features are "active"
/// (evaluated). Features after the cursor are "future" and not included in the
/// current model state. This enables rollback/roll-forward like SolidWorks.
#[derive(Debug)]
pub struct FeatureTree {
    features: Vec<Feature>,
    /// Index of the last active feature (inclusive). None if tree is empty.
    cursor: Option<usize>,
    /// Cached BRep at each feature step. None = needs re-evaluation.
    cache: Vec<Option<BRep>>,
    /// Param hash for each feature. Used for Tier 1 cache validation:
    /// if the hash hasn't changed, the feature doesn't need re-evaluation.
    param_hashes: Vec<Option<u64>>,
    /// BRep fingerprint at each feature step. Used for Tier 2 cache validation:
    /// if the fingerprint hasn't changed after re-evaluation, downstream cache is still valid.
    fingerprints: Vec<Option<BRepFingerprint>>,
    /// Sketch data associated with Sketch features, keyed by feature index.
    /// Stored separately because Sketch is not serializable through FeatureParams.
    pub sketches: HashMap<usize, Sketch>,
    /// Intermediate results: solved profiles from sketch features, keyed by feature index.
    pub sketch_profiles: HashMap<usize, ExtrudeProfile>,
    /// Computed datum planes, keyed by feature index.
    pub datum_planes: HashMap<usize, crate::geometry::surface::plane::Plane>,
    /// Tool bodies for Combine Bodies operations, keyed by feature index.
    pub tool_bodies: HashMap<usize, BRep>,
}

impl FeatureTree {
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
            cursor: None,
            cache: Vec::new(),
            param_hashes: Vec::new(),
            fingerprints: Vec::new(),
            sketches: HashMap::new(),
            sketch_profiles: HashMap::new(),
            datum_planes: HashMap::new(),
            tool_bodies: HashMap::new(),
        }
    }

    /// Add a feature at the end of the tree and advance the cursor.
    pub fn push(&mut self, feature: Feature) {
        self.features.push(feature);
        self.cache.push(None);
        self.param_hashes.push(None);
        self.fingerprints.push(None);
        self.cursor = Some(self.features.len() - 1);
    }

    /// Insert a feature at the current cursor position.
    /// Invalidates all cache entries from the insertion point onward.
    pub fn insert_at_cursor(&mut self, feature: Feature) -> KernelResult<usize> {
        let index = match self.cursor {
            Some(c) => c + 1,
            None => 0,
        };
        self.features.insert(index, feature);
        self.cache.insert(index, None);
        self.param_hashes.insert(index, None);
        self.fingerprints.insert(index, None);
        for i in index..self.cache.len() {
            self.cache[i] = None;
            self.param_hashes[i] = None;
            self.fingerprints[i] = None;
        }
        self.cursor = Some(index);
        Ok(index)
    }

    /// Roll back to just before the feature at `index`.
    pub fn rollback_to(&mut self, index: usize) -> KernelResult<()> {
        if index > self.features.len() {
            return Err(KernelError::InvalidParameter {
                param: "index".into(),
                value: index.to_string(),
            });
        }
        self.cursor = if index == 0 { None } else { Some(index - 1) };
        Ok(())
    }

    /// Roll forward to include all features.
    pub fn roll_forward(&mut self) {
        if !self.features.is_empty() {
            self.cursor = Some(self.features.len() - 1);
        }
    }

    /// Suppress a feature (skip it during evaluation). Persisted to JSON.
    pub fn suppress(&mut self, index: usize) -> KernelResult<()> {
        let feature = self.features.get_mut(index).ok_or_else(|| {
            KernelError::NotFound(format!("Feature at index {}", index))
        })?;
        feature.suppressed = true;
        for i in index..self.cache.len() {
            self.cache[i] = None;
            self.param_hashes[i] = None;
            self.fingerprints[i] = None;
        }
        Ok(())
    }

    /// Unsuppress a feature.
    pub fn unsuppress(&mut self, index: usize) -> KernelResult<()> {
        let feature = self.features.get_mut(index).ok_or_else(|| {
            KernelError::NotFound(format!("Feature at index {}", index))
        })?;
        feature.suppressed = false;
        feature.state = FeatureState::Pending;
        for i in index..self.cache.len() {
            self.cache[i] = None;
            self.param_hashes[i] = None;
            self.fingerprints[i] = None;
        }
        Ok(())
    }

    /// Remove a feature by index.
    pub fn remove_feature(&mut self, index: usize) -> KernelResult<()> {
        if index >= self.features.len() {
            return Err(KernelError::NotFound(format!(
                "Feature at index {}",
                index
            )));
        }
        self.features.remove(index);
        self.cache.remove(index);
        self.param_hashes.remove(index);
        self.fingerprints.remove(index);

        // Clean up associated data (sketches, profiles, datum planes, tool bodies)
        // Remove the entry for this index and shift higher indices down
        self.sketches.remove(&index);
        self.sketch_profiles.remove(&index);
        self.datum_planes.remove(&index);
        self.tool_bodies.remove(&index);

        // Shift keys above the removed index
        for map_index in index..self.features.len() {
            if let Some(v) = self.sketches.remove(&(map_index + 1)) {
                self.sketches.insert(map_index, v);
            }
            if let Some(v) = self.sketch_profiles.remove(&(map_index + 1)) {
                self.sketch_profiles.insert(map_index, v);
            }
            if let Some(v) = self.datum_planes.remove(&(map_index + 1)) {
                self.datum_planes.insert(map_index, v);
            }
            if let Some(v) = self.tool_bodies.remove(&(map_index + 1)) {
                self.tool_bodies.insert(map_index, v);
            }
        }

        // Adjust cursor
        if let Some(c) = self.cursor {
            if index <= c {
                self.cursor = if c == 0 { None } else { Some(c - 1) };
            }
        }

        // Invalidate from the removal point
        if index < self.cache.len() {
            self.invalidate_from(index);
        }

        Ok(())
    }

    /// Rename a feature by index.
    pub fn rename_feature(&mut self, index: usize, name: String) -> KernelResult<()> {
        self.features
            .get_mut(index)
            .ok_or_else(|| KernelError::NotFound(format!("Feature at index {}", index)))?
            .name = name;
        Ok(())
    }

    /// Move a feature from one index to another.
    pub fn move_feature(&mut self, from: usize, to: usize) -> KernelResult<()> {
        if from >= self.features.len() || to >= self.features.len() {
            return Err(KernelError::InvalidParameter {
                param: "index".into(),
                value: format!("from={}, to={}, len={}", from, to, self.features.len()),
            });
        }
        if from == to {
            return Ok(());
        }

        let feature = self.features.remove(from);
        self.features.insert(to, feature);

        let cache_entry = self.cache.remove(from);
        self.cache.insert(to, cache_entry);

        let hash_entry = self.param_hashes.remove(from);
        self.param_hashes.insert(to, hash_entry);

        let fp_entry = self.fingerprints.remove(from);
        self.fingerprints.insert(to, fp_entry);

        // Invalidate from the earliest affected index
        self.invalidate_from(from.min(to));

        Ok(())
    }

    /// Get the active features (up to and including cursor).
    pub fn active_features(&self) -> &[Feature] {
        match self.cursor {
            Some(c) => &self.features[..=c],
            None => &[],
        }
    }

    pub fn len(&self) -> usize {
        self.features.len()
    }

    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn features(&self) -> &[Feature] {
        &self.features
    }

    pub fn features_mut(&mut self) -> &mut Vec<Feature> {
        &mut self.features
    }

    pub fn cache_at(&self, index: usize) -> Option<&BRep> {
        self.cache.get(index).and_then(|c| c.as_ref())
    }

    pub fn set_cache(&mut self, index: usize, brep: BRep) {
        if index < self.cache.len() {
            self.cache[index] = Some(brep);
        }
    }

    /// Clear the cache entry at the given index (set to None).
    pub fn set_cache_none(&mut self, index: usize) {
        if index < self.cache.len() {
            self.cache[index] = None;
            self.param_hashes[index] = None;
            self.fingerprints[index] = None;
        }
    }

    pub fn invalidate_from(&mut self, index: usize) {
        for i in index..self.cache.len() {
            self.cache[i] = None;
            self.param_hashes[i] = None;
            self.fingerprints[i] = None;
        }
    }

    /// Get the param hash for a feature at the given index.
    pub fn param_hash_at(&self, index: usize) -> Option<u64> {
        self.param_hashes.get(index).and_then(|h| *h)
    }

    /// Set the param hash for a feature at the given index.
    pub fn set_param_hash(&mut self, index: usize, hash: u64) {
        if index < self.param_hashes.len() {
            self.param_hashes[index] = Some(hash);
        }
    }

    /// Get the BRep fingerprint for a feature at the given index.
    pub fn fingerprint_at(&self, index: usize) -> Option<&BRepFingerprint> {
        self.fingerprints.get(index).and_then(|f| f.as_ref())
    }

    /// Set the BRep fingerprint for a feature at the given index.
    pub fn set_fingerprint(&mut self, index: usize, fp: BRepFingerprint) {
        if index < self.fingerprints.len() {
            self.fingerprints[index] = Some(fp);
        }
    }
}

impl Default for FeatureTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature_tree::{FeatureKind, FeatureParams};

    fn test_feature(name: &str) -> Feature {
        Feature::new(
            format!("{}-1", name.to_lowercase()),
            name.into(),
            FeatureKind::Extrude,
            FeatureParams::Placeholder,
        )
    }

    #[test]
    fn empty_tree() {
        let tree = FeatureTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.cursor(), None);
        assert_eq!(tree.active_features().len(), 0);
    }

    #[test]
    fn push_advances_cursor() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        assert_eq!(tree.cursor(), Some(0));
        tree.push(test_feature("F2"));
        assert_eq!(tree.cursor(), Some(1));
        assert_eq!(tree.active_features().len(), 2);
    }

    #[test]
    fn rollback_and_forward() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        tree.push(test_feature("F3"));

        tree.rollback_to(1).unwrap();
        assert_eq!(tree.cursor(), Some(0));
        assert_eq!(tree.active_features().len(), 1);

        tree.roll_forward();
        assert_eq!(tree.cursor(), Some(2));
        assert_eq!(tree.active_features().len(), 3);
    }

    #[test]
    fn rollback_to_zero() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.rollback_to(0).unwrap();
        assert_eq!(tree.cursor(), None);
        assert_eq!(tree.active_features().len(), 0);
    }

    #[test]
    fn suppress_feature() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        tree.suppress(0).unwrap();
        assert!(tree.features()[0].suppressed);
        assert!(!tree.features()[0].is_active());
    }

    #[test]
    fn insert_at_cursor() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F3"));
        tree.rollback_to(1).unwrap();
        tree.insert_at_cursor(test_feature("F2")).unwrap();
        assert_eq!(tree.len(), 3);
        assert_eq!(tree.features()[1].name, "F2");
    }

    #[test]
    fn test_remove_feature() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        tree.push(test_feature("F3"));

        // Remove middle feature
        tree.remove_feature(1).unwrap();
        assert_eq!(tree.len(), 2);
        assert_eq!(tree.features()[0].name, "F1");
        assert_eq!(tree.features()[1].name, "F3");
        assert_eq!(tree.cursor(), Some(1));
    }

    #[test]
    fn test_remove_feature_adjusts_cursor() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        assert_eq!(tree.cursor(), Some(1));

        // Remove first feature, cursor should move from 1 to 0
        tree.remove_feature(0).unwrap();
        assert_eq!(tree.cursor(), Some(0));
        assert_eq!(tree.len(), 1);

        // Remove last remaining feature
        tree.remove_feature(0).unwrap();
        assert_eq!(tree.cursor(), None);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_remove_feature_out_of_bounds() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        assert!(tree.remove_feature(5).is_err());
    }

    #[test]
    fn test_rename_feature() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.rename_feature(0, "Renamed".into()).unwrap();
        assert_eq!(tree.features()[0].name, "Renamed");
    }

    #[test]
    fn test_rename_feature_out_of_bounds() {
        let mut tree = FeatureTree::new();
        assert!(tree.rename_feature(0, "X".into()).is_err());
    }

    #[test]
    fn test_move_feature() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        tree.push(test_feature("F3"));

        // Move F3 (index 2) to index 0
        tree.move_feature(2, 0).unwrap();
        assert_eq!(tree.features()[0].name, "F3");
        assert_eq!(tree.features()[1].name, "F1");
        assert_eq!(tree.features()[2].name, "F2");
    }

    #[test]
    fn test_move_feature_same_index() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.move_feature(0, 0).unwrap();
        assert_eq!(tree.features()[0].name, "F1");
    }

    #[test]
    fn test_move_feature_out_of_bounds() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        assert!(tree.move_feature(0, 5).is_err());
        assert!(tree.move_feature(5, 0).is_err());
    }

    #[test]
    fn test_remove_invalidates_cache() {
        let mut tree = FeatureTree::new();
        tree.push(test_feature("F1"));
        tree.push(test_feature("F2"));
        tree.push(test_feature("F3"));

        // Manually set some cache entries
        use crate::topology::BRep;
        tree.set_cache(0, BRep::new());
        tree.set_cache(1, BRep::new());
        tree.set_cache(2, BRep::new());

        assert!(tree.cache_at(0).is_some());
        assert!(tree.cache_at(1).is_some());
        assert!(tree.cache_at(2).is_some());

        // Remove middle feature — cache from index 1 onward should be invalidated
        tree.remove_feature(1).unwrap();
        assert_eq!(tree.len(), 2);
        // Index 0 should still have cache (it was before the removal point)
        assert!(tree.cache_at(0).is_some());
        // Index 1 (formerly index 2) should be invalidated
        assert!(tree.cache_at(1).is_none());
    }
}
