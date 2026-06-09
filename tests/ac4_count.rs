//! AC4: --count 3 returns at most 3 proposals; fewer than count is fine if
//! fewer valid proposals exist.  No padding with invalid or duplicate entries.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;

#[test]
fn ac4_count_ceiling_respected() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    for count in [1usize, 2, 3, 5] {
        let output = proposer::propose(&session, &model, &summary, count, Strategy::Drill);
        assert!(
            output.proposals.len() <= count,
            "count={count}: got {} proposals, expected ≤ {count}",
            output.proposals.len()
        );
    }
}

#[test]
fn ac4_count_max_clamped_at_5_by_caller() {
    // The binary clamps at 5; the lib function respects whatever count it's given
    // We verify that passing count=5 returns ≤5 proposals
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Auto);
    assert!(output.proposals.len() <= 5);
}

#[test]
fn ac4_no_duplicate_proposals() {
    let session = helpers::load_fixture("session_drill.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 5, Strategy::Auto);

    // No two proposals should have the same rationale
    let rationales: Vec<_> = output.proposals.iter().map(|p| &p.rationale).collect();
    let unique_count = rationales.iter().collect::<std::collections::HashSet<_>>().len();
    assert_eq!(rationales.len(), unique_count, "duplicate proposals detected");
}
