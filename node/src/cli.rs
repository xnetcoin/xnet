//! # CLI Definitions
//!
//! Declares the top-level `Cli` struct and the `Subcommand` enum that drives
//! argument parsing via `clap`. Each variant maps one-to-one to a Substrate
//! CLI command (key management, spec building, benchmarking, etc.).

use sc_cli::RunCmd;

/// Root CLI structure for the XNET node binary.
///
/// When no subcommand is given, the node starts in full-node (or validator) mode
/// using the `run` field to configure networking, RPC, and pruning options.
#[derive(Debug, clap::Parser)]
#[command(
	name = "xnet-node",
	author = "XNET Protocol Team",
	about = "The High-Performance Blockchain Node for XNET Protocol.",
	version = "1.0.0",
	propagate_version = true,
	args_conflicts_with_subcommands = true,
	subcommand_negates_reqs = true
)]
pub struct Cli {
	#[command(subcommand)]
	pub subcommand: Option<Subcommand>,

	#[clap(flatten)]
	pub run: RunCmd,
}

/// Available subcommands for the XNET node binary.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
	/// Key management tools — generate, insert, and inspect node and account keys.
	#[command(subcommand)]
	Key(sc_cli::KeySubcommand),

	/// Serialize the chain specification to a JSON file.
	BuildSpec(sc_cli::BuildSpecCmd),

	/// Re-execute and validate a range of blocks from the database.
	CheckBlock(sc_cli::CheckBlockCmd),

	/// Dump a range of finalized blocks to a binary file for offline analysis.
	ExportBlocks(sc_cli::ExportBlocksCmd),

	/// Export the storage trie at a given block height as a chain-spec patch.
	ExportState(sc_cli::ExportStateCmd),

	/// Import blocks from a file produced by `ExportBlocks`.
	ImportBlocks(sc_cli::ImportBlocksCmd),

	/// Wipe the entire chain database. Use with caution — this is irreversible.
	PurgeChain(sc_cli::PurgeChainCmd),

	/// Roll the canonical chain back by N blocks (useful for debugging forks).
	Revert(sc_cli::RevertCmd),

	/// Measure the weight (execution time) of pallets and extrinsics.
	#[command(subcommand)]
	Benchmark(frame_benchmarking_cli::BenchmarkCmd),

	/// Print low-level database column metadata for diagnostics.
	ChainInfo(sc_cli::ChainInfoCmd),
}
