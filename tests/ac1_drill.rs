//! AC1: Given a session where the last MQO used "Store Region" and the model
//! has "Store State" as a child level, a drill proposal is generated with
//! "Store State" in its dimensions.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;

#[test]
fn ac1_drill_produces_store_state() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Drill);

    assert!(!output.proposals.is_empty(), "expected at least one proposal");

    let drill = output
        .proposals
        .iter()
        .find(|p| p.strategy == "drill")
        .expect("expected a drill proposal");

    // The MQO must contain Store State in dimensions
    let dims = drill.mqo["dimensions"]
        .as_array()
        .expect("dimensions must be an array");
    let has_store_state = dims
        .iter()
        .any(|d| d["level"].as_str() == Some("Store State"));
    assert!(
        has_store_state,
        "expected 'Store State' in drill proposal dimensions, got: {dims:?}"
    );

    // Rationale must mention Store State or child
    assert!(
        drill.rationale.contains("Store State") || drill.rationale.contains("drill"),
        "rationale should reference the drill target"
    );
}

#[test]
fn ac1_drill_mqo_has_same_measures() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Drill);

    for p in &output.proposals {
        let measures = p.mqo["measures"].as_array().expect("measures array");
        assert!(!measures.is_empty(), "proposals must have measures");
        let has_revenue = measures
            .iter()
            .any(|m| m["unique_name"].as_str() == Some("sales.revenue"));
        assert!(has_revenue, "expected sales.revenue in proposal measures");
    }
}
