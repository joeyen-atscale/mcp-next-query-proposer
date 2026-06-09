//! AC7: Proposal generation for a 500-entity model with a 20-query session
//! history completes in under 100ms.

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;
use serde_json::{json, Value};
use std::time::Instant;

/// Build a model with `hier_count` hierarchies, each with `levels_per_hier` levels.
fn build_large_model(hier_count: usize, levels_per_hier: usize) -> Value {
    let mut measures: Vec<Value> = Vec::new();
    for i in 0..10 {
        measures.push(json!({"unique_name": format!("sales.measure_{i}"), "label": format!("Measure {i}")}));
    }

    let mut hierarchies: Vec<Value> = Vec::new();
    for h in 0..hier_count {
        let is_time = h == 0;
        let mut levels: Vec<Value> = Vec::new();
        for l in 0..levels_per_hier {
            levels.push(json!({
                "unique_name": format!("dim{h}.level{l}"),
                "name": format!("Level{l}"),
                "depth": l + 1
            }));
        }
        hierarchies.push(json!({
            "unique_name": format!("dim{h}"),
            "is_time": is_time,
            "levels": levels
        }));
    }

    json!({
        "measures": measures,
        "hierarchies": hierarchies
    })
}

/// Build a session with `query_count` MQOs, touching the first `touched_count` entities.
fn build_large_session(query_count: usize, touched: &[String]) -> Value {
    let mut history: Vec<Value> = Vec::new();
    for _i in 0..query_count {
        history.push(json!({
            "mqo": {
                "model": "sales",
                "measures": [{"unique_name": "sales.measure_0"}],
                "dimensions": [{"hierarchy": "dim1", "level": "Level0"}],
                "filters": [],
                "time_intelligence": []
            }
        }));
    }

    let touched_json: Vec<Value> = touched.iter().map(|s| json!(s)).collect();

    json!({
        "session_id": "bench-session",
        "mqo_history": history,
        "touched_entities": touched_json
    })
}

#[test]
fn ac7_perf_under_100ms() {
    // Build a ~500-entity model: 50 hierarchies × 10 levels = 500 levels + 10 measures
    let model = build_large_model(50, 10);

    // Touch only the first 5 entities so there's plenty of frontier
    let touched: Vec<String> = (0..5).map(|i| format!("dim0.level{i}")).collect();

    let session = build_large_session(20, &touched);

    let summary = json!({
        "row_count": 1000,
        "columns": [],
        "sample": [],
        "sample_cap": 20,
        "stats": {
            "dim1.level0": {
                "min": null, "max": null, "sum": null, "mean": null,
                "distinct": 52,
                "top_k": null
            }
        },
        "notes": []
    });

    let start = Instant::now();
    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Auto);
    let elapsed = start.elapsed();

    println!("proposals: {}, elapsed: {:?}", output.proposals.len(), elapsed);

    assert!(
        elapsed.as_millis() < 100,
        "proposal generation took {}ms, expected < 100ms",
        elapsed.as_millis()
    );
}
