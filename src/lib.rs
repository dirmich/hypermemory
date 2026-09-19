pub mod domain;
pub mod storage;
pub mod ingestion;
pub mod extractor;
pub mod temporal;
pub mod search;
pub mod wiki;
pub mod api;
pub mod mcp;
pub mod cli;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
