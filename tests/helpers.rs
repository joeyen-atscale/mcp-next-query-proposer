//! Shared test helpers.

use serde_json::Value;
use std::path::Path;

pub fn load_fixture(name: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name);
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {name}: {e}"));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("failed to parse fixture {name}: {e}"))
}
