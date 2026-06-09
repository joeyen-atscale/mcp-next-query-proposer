//! Shared output types and CLI enum helpers.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::str::FromStr;

// ── CLI enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strategy {
    Drill,
    Compare,
    Pivot,
    Auto,
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Strategy::Drill => write!(f, "drill"),
            Strategy::Compare => write!(f, "compare"),
            Strategy::Pivot => write!(f, "pivot"),
            Strategy::Auto => write!(f, "auto"),
        }
    }
}

impl FromStr for Strategy {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "drill" => Ok(Strategy::Drill),
            "compare" => Ok(Strategy::Compare),
            "pivot" => Ok(Strategy::Pivot),
            "auto" => Ok(Strategy::Auto),
            other => Err(format!("unknown strategy: '{other}'; expected drill|compare|pivot|auto")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Json,
    Human,
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Format::Json => write!(f, "json"),
            Format::Human => write!(f, "human"),
        }
    }
}

impl FromStr for Format {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "json" => Ok(Format::Json),
            "human" => Ok(Format::Human),
            other => Err(format!("unknown format: '{other}'; expected json|human")),
        }
    }
}

// ── Output types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub rank: usize,
    pub strategy: String,
    pub rationale: String,
    pub mqo: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub session_id: String,
    pub proposals: Vec<Proposal>,
}
