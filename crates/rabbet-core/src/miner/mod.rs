//! Repository mining and AST pattern discovery engine.

pub mod ast_skeleton;
pub mod cold_miner;
pub mod pattern_miner;

pub use ast_skeleton::{extract_ast_skeleton, normalize_whitespace_skeleton};
pub use cold_miner::{ColdMinerOptions, extract_synthetic_traces_from_source, mine_local_repo};
pub use pattern_miner::{
    PatternMinerOptions, discover_pattern_candidates, export_dado_pattern_file,
};

