//! mcp-next-query-proposer binary entry point.
//!
//! Reads session state, a model description, and the last dataset handle
//! summary, then proposes typed follow-up MQOs using graph traversal over
//! the model's entity adjacency structure.  No LLM, no network.

use clap::Parser;
use mcp_next_query_proposer::proposer;
use mcp_next_query_proposer::types::{Format, Output, Strategy};
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(
    name = "mcp-next-query-proposer",
    about = "Propose follow-up MQOs from session context"
)]
struct Args {
    /// Path to session.json (serialized SessionState)
    #[arg(long)]
    session: PathBuf,

    /// Path to describe.json (describe_model JSON)
    #[arg(long)]
    model: PathBuf,

    /// Path to last-handle summary.json (DatasetSummary)
    #[arg(long = "last-handle")]
    last_handle: PathBuf,

    /// Number of proposals to return (default 3, max 5)
    #[arg(long, default_value = "3")]
    count: usize,

    /// Traversal strategy: drill|compare|pivot|auto
    #[arg(long, default_value = "auto", value_parser = parse_strategy)]
    strategy: Strategy,

    /// Output format: json|human
    #[arg(long, default_value = "json", value_parser = parse_format)]
    format: Format,
}

fn parse_strategy(s: &str) -> Result<Strategy, String> {
    s.parse()
}

fn parse_format(s: &str) -> Result<Format, String> {
    s.parse()
}

fn main() {
    let args = Args::parse();

    // Clamp count to max 5
    let count = args.count.min(5);

    // Load inputs
    let session_raw = std::fs::read_to_string(&args.session).unwrap_or_else(|e| {
        eprintln!("error reading session file: {e}");
        process::exit(1);
    });
    let model_raw = std::fs::read_to_string(&args.model).unwrap_or_else(|e| {
        eprintln!("error reading model file: {e}");
        process::exit(1);
    });
    let summary_raw = std::fs::read_to_string(&args.last_handle).unwrap_or_else(|e| {
        eprintln!("error reading last-handle file: {e}");
        process::exit(1);
    });

    let session = serde_json::from_str(&session_raw).unwrap_or_else(|e| {
        eprintln!("error parsing session JSON: {e}");
        process::exit(1);
    });
    let model_val = serde_json::from_str(&model_raw).unwrap_or_else(|e| {
        eprintln!("error parsing model JSON: {e}");
        process::exit(1);
    });
    let summary = serde_json::from_str(&summary_raw).unwrap_or_else(|e| {
        eprintln!("error parsing last-handle JSON: {e}");
        process::exit(1);
    });

    let result = proposer::propose(&session, &model_val, &summary, count, args.strategy);

    let output = match args.format {
        Format::Json => serde_json::to_string_pretty(&result).unwrap_or_else(|e| {
            eprintln!("error serializing output: {e}");
            process::exit(1);
        }),
        Format::Human => format_human(&result),
    };

    println!("{output}");
}

fn format_human(output: &Output) -> String {
    let mut lines = Vec::new();
    lines.push(format!("session_id: {}", output.session_id));
    lines.push(format!("{} proposal(s):", output.proposals.len()));
    for p in &output.proposals {
        lines.push(format!(
            "  [{}] strategy={} — {}",
            p.rank, p.strategy, p.rationale
        ));
    }
    lines.join("\n")
}
