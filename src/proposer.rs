//! Core proposal engine.
//!
//! Given session state, model description, and last dataset summary,
//! builds up to `count` ranked MQO proposals using the selected strategy.

use serde_json::{json, Value};

use crate::model;
use crate::session;
use crate::types::{Output, Proposal, Strategy};
use crate::validate;

// ── Strategy selection ─────────────────────────────────────────────────────

/// Determine the effective strategy when `Strategy::Auto` is requested.
///
/// Auto logic (from PRD):
/// - If last handle has a column with distinct_values > 10 and there are
///   unvisited child levels → drill
/// - If no time_intelligence in prior MQOs → compare
/// - otherwise → pivot
fn select_auto_strategy(
    session: &Value,
    model: &Value,
    summary: &Value,
) -> Strategy {
    let touched = session::touched_entities(session);
    let unvisited = model::unvisited_levels(model, &touched);

    // Check for high-cardinality column in summary
    let has_high_cardinality = summary["stats"]
        .as_object()
        .map(|stats| {
            stats.values().any(|s| {
                s["distinct"].as_u64().unwrap_or(0) > 10
            })
        })
        .unwrap_or(false);

    let has_unvisited_children = !unvisited.is_empty();

    if has_high_cardinality && has_unvisited_children {
        return Strategy::Drill;
    }

    if !session::has_time_intelligence(session) {
        // Check that the model actually has a time hierarchy
        if model::first_time_hierarchy(model).is_some() {
            return Strategy::Compare;
        }
    }

    Strategy::Pivot
}

// ── Proposal builders ──────────────────────────────────────────────────────

/// Build drill proposals: deepen one level in the current dimensions.
fn build_drill_proposals(
    session: &Value,
    model: &Value,
    count: usize,
) -> Vec<Proposal> {
    let measures = session::last_measures(session);
    let dims = session::last_dimensions(session);
    let model_name = session::last_model_name(session);
    let touched = session::touched_entities(session);

    if measures.is_empty() {
        return vec![];
    }

    let mut proposals = Vec::new();
    let mut rank = 1;

    // For each current dimension, try to find a child level
    for (hierarchy, level_name) in &dims {
        if proposals.len() >= count {
            break;
        }

        if let Some(child) = model::child_level(model, hierarchy, level_name) {
            let child_name = child["name"].as_str().unwrap_or("").to_string();
            let child_unique = child["unique_name"].as_str().unwrap_or("").to_string();

            // Skip if already touched
            if touched.contains(&child_unique) {
                continue;
            }
            if child_name.is_empty() {
                continue;
            }

            // Build new dimensions: keep all existing, replace current with child
            let new_dims: Vec<Value> = dims
                .iter()
                .map(|(h, l)| {
                    if h == hierarchy && l == level_name {
                        json!({"hierarchy": h, "level": child_name})
                    } else {
                        json!({"hierarchy": h, "level": l})
                    }
                })
                .collect();

            let measure_refs: Vec<Value> = measures
                .iter()
                .map(|m| json!({"unique_name": m}))
                .collect();

            let mqo = json!({
                "model": model_name,
                "measures": measure_refs,
                "dimensions": new_dims,
                "filters": [],
                "time_intelligence": []
            });

            if validate::validate(&mqo).is_err() {
                continue;
            }

            proposals.push(Proposal {
                rank,
                strategy: "drill".to_string(),
                rationale: format!(
                    "High cardinality on '{level_name}'; drill to '{child_name}'"
                ),
                mqo,
            });
            rank += 1;
        }
    }

    // If no dimension drill worked, try an unvisited level from any hierarchy
    if proposals.is_empty() {
        let unvisited = model::unvisited_levels(model, &touched);
        for level in unvisited.iter().take(count) {
            let measure_refs: Vec<Value> =
                measures.iter().map(|m| json!({"unique_name": m})).collect();

            let new_dims = vec![json!({"hierarchy": level.hierarchy, "level": level.name})];

            let mqo = json!({
                "model": model_name,
                "measures": measure_refs,
                "dimensions": new_dims,
                "filters": [],
                "time_intelligence": []
            });

            if validate::validate(&mqo).is_err() {
                continue;
            }

            proposals.push(Proposal {
                rank,
                strategy: "drill".to_string(),
                rationale: format!(
                    "Unvisited level '{}' in hierarchy '{}'",
                    level.name, level.hierarchy
                ),
                mqo,
            });
            rank += 1;

            if proposals.len() >= count {
                break;
            }
        }
    }

    proposals
}

/// Build compare proposals: add time_intelligence to the current query.
fn build_compare_proposals(
    session: &Value,
    model: &Value,
    count: usize,
) -> Vec<Proposal> {
    let measures = session::last_measures(session);
    let dims = session::last_dimensions(session);
    let model_name = session::last_model_name(session);

    if measures.is_empty() {
        return vec![];
    }

    let Some(time_hierarchy) = model::first_time_hierarchy(model) else {
        return vec![];
    };
    let time_level = model::coarsest_level(model, &time_hierarchy)
        .unwrap_or_else(|| "Year".to_string());

    let measure_refs: Vec<Value> = measures.iter().map(|m| json!({"unique_name": m})).collect();
    let dim_vals: Vec<Value> = dims
        .iter()
        .map(|(h, l)| json!({"hierarchy": h, "level": l}))
        .collect();

    let mut proposals = Vec::new();

    // Prior period
    if proposals.len() < count {
        let mqo = json!({
            "model": model_name,
            "measures": measure_refs.clone(),
            "dimensions": dim_vals.clone(),
            "filters": [],
            "time_intelligence": [{"op": "prior_period"}]
        });
        if validate::validate(&mqo).is_ok() {
            proposals.push(Proposal {
                rank: proposals.len() + 1,
                strategy: "compare".to_string(),
                rationale: format!(
                    "No time intelligence in prior MQOs; add prior_period comparison on '{time_hierarchy}'"
                ),
                mqo,
            });
        }
    }

    // YoY if count allows
    if proposals.len() < count {
        let mut dims_with_time = dim_vals.clone();
        dims_with_time.push(json!({"hierarchy": time_hierarchy, "level": time_level}));
        let mqo = json!({
            "model": model_name,
            "measures": measure_refs.clone(),
            "dimensions": dims_with_time,
            "filters": [],
            "time_intelligence": [{"op": "yoy"}]
        });
        if validate::validate(&mqo).is_ok() {
            proposals.push(Proposal {
                rank: proposals.len() + 1,
                strategy: "compare".to_string(),
                rationale: format!(
                    "Year-over-year comparison using '{time_hierarchy}'"
                ),
                mqo,
            });
        }
    }

    proposals
}

/// Build pivot proposals: swap/add a dimension from the unvisited frontier.
fn build_pivot_proposals(
    session: &Value,
    model: &Value,
    count: usize,
) -> Vec<Proposal> {
    let measures = session::last_measures(session);
    let dims = session::last_dimensions(session);
    let model_name = session::last_model_name(session);
    let touched = session::touched_entities(session);

    if measures.is_empty() {
        return vec![];
    }

    let measure_refs: Vec<Value> = measures.iter().map(|m| json!({"unique_name": m})).collect();

    // Find unvisited non-time levels for pivoting
    let candidates: Vec<model::Level> = model::unvisited_levels(model, &touched)
        .into_iter()
        .filter(|l| !l.is_time)
        .collect();

    let mut proposals = Vec::new();

    for candidate in candidates.iter().take(count) {
        // Build new dimension set: keep all existing, add/swap the candidate
        // Strategy: if existing dims already include this hierarchy, replace; else append
        let already_in = dims.iter().any(|(h, _)| h == &candidate.hierarchy);

        let new_dims: Vec<Value> = if already_in {
            dims.iter()
                .map(|(h, l)| {
                    if h == &candidate.hierarchy {
                        json!({"hierarchy": h, "level": candidate.name})
                    } else {
                        json!({"hierarchy": h, "level": l})
                    }
                })
                .collect()
        } else {
            let mut d: Vec<Value> = dims
                .iter()
                .map(|(h, l)| json!({"hierarchy": h, "level": l}))
                .collect();
            d.push(json!({"hierarchy": candidate.hierarchy, "level": candidate.name}));
            d
        };

        let mqo = json!({
            "model": model_name,
            "measures": measure_refs.clone(),
            "dimensions": new_dims,
            "filters": [],
            "time_intelligence": []
        });

        if validate::validate(&mqo).is_err() {
            continue;
        }

        proposals.push(Proposal {
            rank: proposals.len() + 1,
            strategy: "pivot".to_string(),
            rationale: format!(
                "Pivot to adjacent dimension '{}' (hierarchy: '{}')",
                candidate.name, candidate.hierarchy
            ),
            mqo,
        });
    }

    proposals
}

// ── Main entry point ───────────────────────────────────────────────────────

/// Produce up to `count` ranked MQO proposals.
pub fn propose(
    session: &Value,
    model: &Value,
    summary: &Value,
    count: usize,
    strategy: Strategy,
) -> Output {
    let session_id = session::session_id(session);

    let effective_strategy = match strategy {
        Strategy::Auto => select_auto_strategy(session, model, summary),
        other => other,
    };

    let mut proposals = match effective_strategy {
        Strategy::Drill => build_drill_proposals(session, model, count),
        Strategy::Compare => build_compare_proposals(session, model, count),
        Strategy::Pivot => build_pivot_proposals(session, model, count),
        Strategy::Auto => unreachable!("auto already resolved"),
    };

    // Enforce count ceiling (validation may have dropped some)
    proposals.truncate(count);

    // Re-rank sequentially after truncation
    for (i, p) in proposals.iter_mut().enumerate() {
        p.rank = i + 1;
    }

    Output { session_id, proposals }
}
