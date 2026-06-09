//! AC5: --strategy drill produces only drill proposals; --strategy compare only
//! compare; --strategy pivot only pivot.  Mixed sessions work with --strategy auto.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;

#[test]
fn ac5_strategy_drill_only() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Drill);
    for p in &output.proposals {
        assert_eq!(p.strategy, "drill", "expected only drill, got: {}", p.strategy);
    }
}

#[test]
fn ac5_strategy_compare_only() {
    let session = helpers::load_fixture("session_compare.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_low_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Compare);
    for p in &output.proposals {
        assert_eq!(p.strategy, "compare", "expected only compare, got: {}", p.strategy);
    }
}

#[test]
fn ac5_strategy_pivot_only() {
    let session = helpers::load_fixture("session_pivot.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_low_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Pivot);
    for p in &output.proposals {
        assert_eq!(p.strategy, "pivot", "expected only pivot, got: {}", p.strategy);
    }
}

#[test]
fn ac5_auto_produces_proposals() {
    // Auto should produce at least one proposal from a non-exhausted session
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Auto);
    assert!(
        !output.proposals.is_empty(),
        "auto strategy should produce proposals for a non-exhausted session"
    );
}
