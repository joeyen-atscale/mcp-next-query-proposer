//! Session state extraction helpers.
//!
//! The session JSON shape (inlined minimal):
//! ```json
//! {
//!   "session_id": "abc-123",
//!   "mqo_history": [
//!     {
//!       "mqo": {
//!         "model": "sales",
//!         "measures": [{"unique_name": "sales.revenue"}],
//!         "dimensions": [{"hierarchy": "geo.store", "level": "Store Region"}],
//!         "filters": [],
//!         "time_intelligence": []
//!       }
//!     }
//!   ],
//!   "touched_entities": ["sales.revenue", "geo.store.Store Region"]
//! }
//! ```

use serde_json::Value;
use std::collections::HashSet;

/// Extract the session_id string (default "unknown").
pub fn session_id(session: &Value) -> String {
    session["session_id"]
        .as_str()
        .unwrap_or("unknown")
        .to_string()
}

/// Extract the set of touched entity unique_names.
pub fn touched_entities(session: &Value) -> HashSet<String> {
    let mut set = HashSet::new();
    if let Some(arr) = session["touched_entities"].as_array() {
        for v in arr {
            if let Some(s) = v.as_str() {
                set.insert(s.to_string());
            }
        }
    }
    set
}

/// Return the last MQO value from history, if any.
pub fn last_mqo(session: &Value) -> Option<&Value> {
    session["mqo_history"].as_array()?.last().map(|entry| &entry["mqo"])
}

/// Return the measures from the last MQO as unique_name strings.
pub fn last_measures(session: &Value) -> Vec<String> {
    let Some(mqo) = last_mqo(session) else { return vec![] };
    mqo["measures"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m["unique_name"].as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

/// Return the dimension levels from the last MQO as (hierarchy, level) pairs.
pub fn last_dimensions(session: &Value) -> Vec<(String, String)> {
    let Some(mqo) = last_mqo(session) else { return vec![] };
    mqo["dimensions"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|d| {
                    let h = d["hierarchy"].as_str()?;
                    let l = d["level"].as_str()?;
                    Some((h.to_string(), l.to_string()))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Return true if any prior MQO has time_intelligence entries.
pub fn has_time_intelligence(session: &Value) -> bool {
    let Some(history) = session["mqo_history"].as_array() else { return false };
    history.iter().any(|entry| {
        entry["mqo"]["time_intelligence"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false)
    })
}

/// Return the model name from the last MQO (default "").
pub fn last_model_name(session: &Value) -> String {
    last_mqo(session)
        .and_then(|m| m["model"].as_str())
        .unwrap_or("")
        .to_string()
}
