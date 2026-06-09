//! AC3: All proposed MQOs validate against structural schema; invalid proposals
//! are dropped before output.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;
use mcp_next_query_proposer::validate;

#[test]
fn ac3_all_proposals_are_valid() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    for strategy in [Strategy::Drill, Strategy::Compare, Strategy::Pivot, Strategy::Auto] {
        let output = proposer::propose(&session, &model, &summary, 5, strategy);
        for p in &output.proposals {
            assert!(
                validate::validate(&p.mqo).is_ok(),
                "proposal with strategy={} failed validation: {:?}\nmqo={}",
                p.strategy,
                validate::validate(&p.mqo),
                serde_json::to_string_pretty(&p.mqo).unwrap()
            );
        }
    }
}

#[test]
fn ac3_validate_rejects_missing_model() {
    use serde_json::json;
    let bad = json!({"measures": [{"unique_name": "x"}], "dimensions": []});
    assert!(validate::validate(&bad).is_err());
}

#[test]
fn ac3_validate_rejects_empty_measures() {
    use serde_json::json;
    let bad = json!({"model": "sales", "measures": [], "dimensions": []});
    assert!(validate::validate(&bad).is_err());
}

#[test]
fn ac3_validate_accepts_minimal_mqo() {
    use serde_json::json;
    let good = json!({
        "model": "sales",
        "measures": [{"unique_name": "sales.revenue"}],
        "dimensions": []
    });
    assert!(validate::validate(&good).is_ok());
}
