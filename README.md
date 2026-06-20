# mcp-next-query-proposer

A CLI that reads where a query session has been and proposes the follow-up queries worth running next — as typed, bound-ready MQOs an agent can submit directly, not as English suggestions.

## Why it exists

When an agent asks "revenue by region," the analytically useful next moves are predictable: drill into the level with the most detail, compare against the prior period, or pivot to an adjacent dimension. An LLM can phrase those suggestions, but it has to phrase them — turning "compare to last year" back into a valid query object is exactly the step that goes wrong, and a wrong object costs a round trip to find out.

This tool skips the round trip. It reads the session history, the model's hierarchy graph, and the shape of the last result, then emits the next MQOs already structured the way `query_multidimensional` expects them. The proposals are deterministic graph traversal over the model — no LLM, no network call — so the same session always yields the same suggestions, and every emitted MQO has already passed structural validation.

## Install

The repo has a binary target; build it with cargo.

```sh
cargo install --path .
```

Or build in place and copy the binary where you want it:

```sh
cargo build --release
cp target/release/mcp-next-query-proposer ~/.local/bin/
```

## Quickstart

It takes three JSON inputs — the session so far, the model description, and a summary of the last result handle — and prints ranked proposals.

```sh
mcp-next-query-proposer \
  --session     session.json   \
  --model       describe.json  \
  --last-handle summary.json
```

Run against the bundled fixtures to see a drill proposal:

```sh
mcp-next-query-proposer \
  --session     tests/fixtures/session_drill.json \
  --model       tests/fixtures/model.json \
  --last-handle tests/fixtures/summary_high_cardinality.json
```

```json
{
  "session_id": "sess-drill-001",
  "proposals": [
    {
      "rank": 1,
      "strategy": "drill",
      "rationale": "High cardinality on 'Store Region'; drill to 'Store State'",
      "mqo": {
        "model": "sales",
        "measures": [{ "unique_name": "sales.revenue" }],
        "dimensions": [{ "hierarchy": "geo.store", "level": "Store State" }],
        "filters": [],
        "time_intelligence": []
      }
    }
  ]
}
```

`--format human` collapses each proposal to a one-line rank, strategy, and rationale. When the session has already visited every level the model offers, the tool says so honestly and returns an empty `proposals` list — it does not invent a move.

### Flags

| Flag           | Meaning                                                   | Default  |
| -------------- | --------------------------------------------------------- | -------- |
| `--session`    | Session state JSON (`mqo_history` + `touched_entities`)   | required |
| `--model`      | `describe_model` JSON (measures + hierarchies with levels)| required |
| `--last-handle`| Dataset summary JSON (per-column stats incl. `distinct`)  | required |
| `--count`      | Maximum proposals to return; clamped to 5                 | 3        |
| `--strategy`   | `drill`, `compare`, `pivot`, or `auto`                    | `auto`   |
| `--format`     | `json` or `human`                                         | `json`   |

## How it works

Three strategies, one selector. Each strategy is a traversal over the model's hierarchy levels, seeded by the last MQO in the session:

| Strategy  | What it does                                                              |
| --------- | ------------------------------------------------------------------------- |
| `drill`   | Deepens one level in the current hierarchy (Region → State → City)        |
| `compare` | Adds a `prior_period` comparison, and a year-over-year proposal if room   |
| `pivot`   | Swaps in or adds an adjacent dimension the session hasn't touched yet      |

Under `--strategy auto`, the selector reads the situation and picks one:

- **drill** if the last result has a column with more than 10 distinct values and the model still has unvisited child levels — high cardinality is the signal that detail is hiding under the current grain;
- else **compare** if no prior MQO used time intelligence and the model has a time hierarchy;
- else **pivot**.

Two invariants hold regardless of strategy. Levels already in `touched_entities` are never proposed again, so the session keeps moving forward instead of circling. And every candidate MQO is structurally validated — model present and non-empty, measures present and non-empty with `unique_name` on each, dimensions an array — before it earns a place in the output. Candidates that fail are dropped, and the surviving proposals are re-ranked so ranks stay contiguous.

### Input shapes

`--session` carries the history and the set of entities already seen:

```json
{
  "session_id": "abc-123",
  "mqo_history": [
    {
      "mqo": {
        "model": "sales",
        "measures": [{ "unique_name": "sales.revenue" }],
        "dimensions": [{ "hierarchy": "geo.store", "level": "Store Region" }],
        "filters": [],
        "time_intelligence": []
      }
    }
  ],
  "touched_entities": ["sales.revenue", "geo.store.Store Region"]
}
```

`--model` is the `describe_model` output — measures, and hierarchies whose levels carry a `depth` (1 is coarsest) and an `is_time` flag on the hierarchy:

```json
{
  "measures": [{ "unique_name": "sales.revenue", "label": "Revenue" }],
  "hierarchies": [
    {
      "unique_name": "geo.store",
      "is_time": false,
      "levels": [
        { "unique_name": "geo.store.Store Region", "name": "Store Region", "depth": 1 },
        { "unique_name": "geo.store.Store State",  "name": "Store State",  "depth": 2 }
      ]
    },
    {
      "unique_name": "time.calendar",
      "is_time": true,
      "levels": [{ "unique_name": "time.calendar.Year", "name": "Year", "depth": 1 }]
    }
  ]
}
```

`--last-handle` is a dataset summary; the proposer reads only `stats.<column>.distinct` to detect high cardinality:

```json
{
  "row_count": 500,
  "stats": {
    "geo.store.Store Region": { "distinct": 52 }
  }
}
```

The fields shown are the ones the tool actually reads; the real summaries carry more, and extra fields are ignored.

## Where it fits

Part of the MQO fleet around `mqo-mcp`. This proposer answers "what should the agent ask next"; it pairs with the dataset-handle tooling that produces the `--last-handle` summary and the `describe_model` output that feeds `--model`. It does no I/O of its own beyond reading the three files and printing — wiring it to a live session is the caller's job.

## Status

Early and self-contained. The strategy set is the three above; validation is structural, not semantic — it confirms an MQO is well-formed, not that the named measures and levels resolve against the model. Coverage is 19 integration tests over the drill / compare / pivot / count / strategy-selection / exhausted-frontier behaviors (AC1–AC6), plus unit tests on the validator.

## License

MIT OR Apache-2.0.
