use std::collections::HashMap;

use crate::error::{KernelError, KernelResult};
use crate::operations::extrude::ExtrudeProfile;
use crate::sketch::Sketch;
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
        for i in index..self.cache.len() {
            self.cache[i] = None;
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
        }
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

    pub fn invalidate_from(&mut self, index: usize) {
        for i in index..self.cache.len() {
            self.cache[i] = None;
        }
    }

    /// Update a feature's parameters and invalidate caches from that index onward.
    pub fn update_params(
        &mut self,
        index: usize,
        params: super::params::FeatureParams,
    ) -> KernelResult<()> {
        let feature = self.features.get_mut(index).ok_or_else(|| {
            KernelError::NotFound(format!("Feature at index {}", index))
        })?;
        feature.params = params;
        self.invalidate_from(index);
        Ok(())
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
}
