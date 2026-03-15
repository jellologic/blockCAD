use std::collections::HashMap;

use crate::solver::variable::VariableId;

use super::entity::SketchEntityId;

/// Maps sketch entity IDs to their corresponding solver variable IDs.
/// Each Point gets 2 variables (x, y). Circles get an additional radius variable.
#[derive(Debug)]
pub struct VariableMap {
    /// For each entity: the list of variable IDs allocated for it.
    /// - Point: [x, y]
    /// - Circle: [radius] (center point vars tracked under the center entity)
    /// - Line/Arc/Spline: empty (they reference other entities' points)
    map: HashMap<SketchEntityId, Vec<VariableId>>,
}

impl VariableMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn insert(&mut self, entity_id: SketchEntityId, var_ids: Vec<VariableId>) {
        self.map.insert(entity_id, var_ids);
    }

    /// Get the (x, y) variable IDs for a Point entity.
    pub fn point_vars(&self, entity_id: SketchEntityId) -> Option<(VariableId, VariableId)> {
        let vars = self.map.get(&entity_id)?;
        if vars.len() >= 2 {
            Some((vars[0], vars[1]))
        } else {
            None
        }
    }

    /// Get the radius variable ID for a Circle entity.
    pub fn circle_radius_var(&self, entity_id: SketchEntityId) -> Option<VariableId> {
        let vars = self.map.get(&entity_id)?;
        if vars.len() == 1 {
            Some(vars[0])
        } else {
            None
        }
    }

    /// Get all variable IDs for a given entity.
    pub fn get(&self, entity_id: SketchEntityId) -> Option<&Vec<VariableId>> {
        self.map.get(&entity_id)
    }
}
