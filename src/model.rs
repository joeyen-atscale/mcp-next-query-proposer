//! Model description extraction helpers.
//!
//! Inlined minimal describe_model JSON shape:
//! ```json
//! {
//!   "measures": [
//!     {"unique_name": "sales.revenue", "label": "Revenue"},
//!     {"unique_name": "sales.units", "label": "Units Sold"}
//!   ],
//!   "hierarchies": [
//!     {
//!       "unique_name": "geo.store",
//!       "levels": [
//!         {"unique_name": "geo.store.Store Region", "name": "Store Region", "depth": 1},
//!         {"unique_name": "geo.store.Store State",  "name": "Store State",  "depth": 2},
//!         {"unique_name": "geo.store.Store City",   "name": "Store City",   "depth": 3}
//!       ]
//!     },
//!     {
//!       "unique_name": "time.calendar",
//!       "is_time": true,
//!       "levels": [
//!         {"unique_name": "time.calendar.Year",    "name": "Year",    "depth": 1},
//!         {"unique_name": "time.calendar.Quarter", "name": "Quarter", "depth": 2},
//!         {"unique_name": "time.calendar.Month",   "name": "Month",   "depth": 3}
//!       ]
//!     }
//!   ]
//! }
//! ```

use serde_json::Value;

/// A resolved hierarchy level with its parent hierarchy.
#[derive(Debug, Clone)]
pub struct Level {
    /// Fully-qualified unique_name for this level.
    pub unique_name: String,
    /// Short name (e.g. "Store State").
    pub name: String,
    /// Depth within the hierarchy (1 = coarsest).
    pub depth: u64,
    /// Parent hierarchy unique_name.
    pub hierarchy: String,
    /// True if this hierarchy is a time dimension.
    pub is_time: bool,
}

/// All levels from all hierarchies in the model.
pub fn all_levels(model: &Value) -> Vec<Level> {
    let mut levels = Vec::new();
    let Some(hierarchies) = model["hierarchies"].as_array() else {
        return levels;
    };
    for hier in hierarchies {
        let hier_name = hier["unique_name"].as_str().unwrap_or("").to_string();
        let is_time = hier["is_time"].as_bool().unwrap_or(false);
        if let Some(lvls) = hier["levels"].as_array() {
            for l in lvls {
                let unique_name = l["unique_name"].as_str().unwrap_or("").to_string();
                let name = l["name"].as_str().unwrap_or("").to_string();
                let depth = l["depth"].as_u64().unwrap_or(1);
                if !unique_name.is_empty() {
                    levels.push(Level {
                        unique_name,
                        name,
                        depth,
                        hierarchy: hier_name.clone(),
                        is_time,
                    });
                }
            }
        }
    }
    levels
}

/// Return the level immediately deeper than `level_name` in the same hierarchy,
/// or None if already at the deepest.
pub fn child_level<'a>(model: &'a Value, hierarchy: &str, level_name: &str) -> Option<&'a Value> {
    let hierarchies = model["hierarchies"].as_array()?;
    let hier = hierarchies.iter().find(|h| h["unique_name"].as_str() == Some(hierarchy))?;
    let lvls = hier["levels"].as_array()?;

    // Find the depth of the current level
    let current_depth = lvls
        .iter()
        .find(|l| l["name"].as_str() == Some(level_name) || l["unique_name"].as_str() == Some(level_name))
        .and_then(|l| l["depth"].as_u64())?;

    // Return the level at current_depth + 1
    lvls.iter().find(|l| l["depth"].as_u64() == Some(current_depth + 1))
}

/// Return all measures as unique_name strings.
pub fn all_measures(model: &Value) -> Vec<String> {
    model["measures"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["unique_name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Return all dimension levels that are in the model but not in `touched`.
pub fn unvisited_levels<'a>(model: &'a Value, touched: &std::collections::HashSet<String>) -> Vec<Level> {
    all_levels(model)
        .into_iter()
        .filter(|l| !touched.contains(&l.unique_name))
        .collect()
}

/// Return the first time hierarchy in the model, if any.
pub fn first_time_hierarchy(model: &Value) -> Option<String> {
    model["hierarchies"].as_array().and_then(|arr| {
        arr.iter()
            .find(|h| h["is_time"].as_bool().unwrap_or(false))
            .and_then(|h| h["unique_name"].as_str())
            .map(String::from)
    })
}

/// Return the coarsest level (depth=1) of a hierarchy.
pub fn coarsest_level(model: &Value, hierarchy: &str) -> Option<String> {
    let hierarchies = model["hierarchies"].as_array()?;
    let hier = hierarchies.iter().find(|h| h["unique_name"].as_str() == Some(hierarchy))?;
    let lvls = hier["levels"].as_array()?;
    let min_depth = lvls.iter().filter_map(|l| l["depth"].as_u64()).min()?;
    lvls.iter()
        .find(|l| l["depth"].as_u64() == Some(min_depth))
        .and_then(|l| l["name"].as_str())
        .map(String::from)
}
