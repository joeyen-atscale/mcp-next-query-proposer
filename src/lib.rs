//! Library interface for mcp-next-query-proposer.
//!
//! Exposes the proposer, validate, and types modules for integration testing.

#![forbid(unsafe_code)]
#![deny(clippy::all)]
#![allow(clippy::module_name_repetitions)]

pub mod model;
pub mod proposer;
pub mod session;
pub mod types;
pub mod validate;
