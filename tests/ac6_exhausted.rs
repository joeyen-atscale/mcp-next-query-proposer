//! AC6: If touched_entities contains all model entities, output is
//! {"proposals": []} with exit 0, not an error.

mod helpers;

use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::Strategy;

#[test]
fn ac6_exhausted_session_returns_empty_proposals() {
    let session = helpers::load_fixture("session_exhausted.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Auto);
    assert_eq!(
        output.proposals.len(),
        0,
        "fully-explored session should yield no proposals"
    );
}

#[test]
fn ac6_exhausted_returns_correct_session_id() {
    let session = helpers::load_fixture("session_exhausted.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    let output = proposer::propose(&session, &model, &summary, 3, Strategy::Auto);
    assert_eq!(output.session_id, "sess-exhausted-001");
}

#[test]
fn ac6_exhausted_drill_also_empty() {
    let session = helpers::load_fixture("session_exhausted.json");
    let model = helpers::load_fixture("model.json");
    let summary = helpers::load_fixture("summary_high_cardinality.json");

    for strategy in [Strategy::Drill, Strategy::Pivot] {
        let output = proposer::propose(&session, &model, &summary, 3, strategy);
        assert_eq!(
            output.proposals.len(),
            0,
            "exhausted session with {:?} should yield no proposals",
            strategy
        );
    }
}
