//! AC2: Given a session with no time-intelligence in any prior MQO and a model
//! with a "Sale Date" / time dimension, a compare proposal is generated that
//! adds a time_intelligence prior_period element.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;

#[test]
fn ac2_compare_adds_prior_period() {
    let session = helpers::load_fixture("session_compare.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_low_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Compare);

    assert!(!output.proposals.is_empty(), "expected at least one compare proposal");

    let compare = output
        .proposals
        .iter()
        .find(|p| p.strategy == "compare")
        .expect("expected a compare proposal");

    let ti = compare.mqo["time_intelligence"]
        .as_array()
        .expect("time_intelligence must be an array");

    assert!(!ti.is_empty(), "compare proposal must have time_intelligence entries");

    let has_prior_period = ti.iter().any(|t| t["op"].as_str() == Some("prior_period"));
    assert!(
        has_prior_period,
        "expected prior_period time_intelligence op, got: {ti:?}"
    );
}

#[test]
fn ac2_auto_selects_compare_when_no_time_intel() {
    let session = helpers::load_fixture("session_compare.json");
    let model = helpers::load_fixture("model.json");
    // Low cardinality → should not trigger drill → should trigger compare
    let summary = helpers::load_fixture("summary_low_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Auto);

    let has_compare = output.proposals.iter().any(|p| p.strategy == "compare");
    assert!(has_compare, "auto should select compare when no time intel and low cardinality");
}
