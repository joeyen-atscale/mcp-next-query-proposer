# mcp-next-query-proposer

Propose analytically valuable follow-up MQOs from session context.

When an agent asks "revenue by region," the obvious follow-up questions are "drill into the highest region," "compare to prior period," and "break down by product category." This CLI reads the session state and the model's entity adjacency graph and proposes the top N follow-up MQOs as **typed, bound-ready objects** — not natural language suggestions, but actual MQOs the agent can submit directly to `query_multidimensional`.

No LLM, no network — proposals are deterministic graph-traversal outputs.

## Usage

```
mcp-next-query-proposer \
  --session   session.json     \
  --model     describe.json    \
  --last-handle summary.json   \
  [--count 3]                  \
  [--strategy drill|compare|pivot|auto] \
  [--format json|human]
```

### Arguments

| Flag | Description | Default |
|------|-------------|---------|
| `--session` | SessionState JSON (mqo_history + touched_entities) | required |
| `--model` | describe_model JSON (measures + hierarchies with levels) | required |
| `--last-handle` | DatasetSummary JSON (per-column stats including distinct) | required |
| `--count` | Max proposals to return (clamped to 5) | 3 |
| `--strategy` | `drill`, `compare`, `pivot`, or `auto` | `auto` |
| `--format` | `json` or `human` | `json` |

## Output

```json
{
  "session_id": "abc-123",
  "proposals": [
    {
      "rank": 1,
      "strategy": "drill",
      "rationale": "High cardinality on 'Store Region'; drill to 'Store State'",
      "mqo": {
        "model": "sales",
        "measures": [{"unique_name": "sales.revenue"}],
        "dimensions": [{"hierarchy": "geo.store", "level": "Store State"}],
        "filters": [],
        "time_intelligence": []
      }
    }
  ]
}
```

## Input shapes

### session.json

```json
{
  "session_id": "abc-123",
  "mqo_history": [
    {
      "mqo": {
        "model": "sales",
        "measures": [{"unique_name": "sales.revenue"}],
        "dimensions": [{"hierarchy": "geo.store", "level": "Store Region"}],
        "filters": [],
        "time_intelligence": []
      }
    }
  ],
  "touched_entities": ["sales.revenue", "geo.store.Store Region"]
}
```

### describe.json

```json
{
  "measures": [
    {"unique_name": "sales.revenue", "label": "Revenue"}
  ],
  "hierarchies": [
    {
      "unique_name": "geo.store",
      "is_time": false,
      "levels": [
        {"unique_name": "geo.store.Store Region", "name": "Store Region", "depth": 1},
        {"unique_name": "geo.store.Store State",  "name": "Store State",  "depth": 2}
      ]
    },
    {
      "unique_name": "time.calendar",
      "is_time": true,
      "levels": [
        {"unique_name": "time.calendar.Year", "name": "Year", "depth": 1}
      ]
    }
  ]
}
```

### summary.json (DatasetSummary)

```json
{
  "row_count": 500,
  "columns": [...],
  "sample": [],
  "sample_cap": 20,
  "stats": {
    "geo.store.Store Region": {"distinct": 52, ...}
  },
  "notes": []
}
```

## Strategy logic

| Strategy | When (auto) | What it does |
|----------|-------------|--------------|
| `drill` | Last handle has a column with distinct > 10 AND unvisited child levels exist | Deepens one level in the current dimension hierarchy |
| `compare` | No time_intelligence in prior MQOs AND model has a time dimension | Adds `prior_period` (and optionally YoY) time_intelligence |
| `pivot` | Otherwise | Swaps or adds an adjacent unvisited dimension from the frontier |

## Building

```sh
cargo build --release
# Binary at: target/release/mcp-next-query-proposer
```

## Testing

```sh
cargo test
# 24 integration tests covering AC1–AC7
```

## Install

```sh
cp target/release/mcp-next-query-proposer ~/.local/bin/
```
