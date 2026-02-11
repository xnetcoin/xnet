use sc_cli::RunCmd;

/// XNET CLI Configuration
#[derive(Debug, clap::Parser)]
#[command(
    name = "xnet-node",
    author = "XNET Protocol Team",
    about = " The High-Performance Blockchain Node for XNET Protocol.",
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

#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// 🔑 Key management tools (Wallet & Node keys)
    #[command(subcommand)]
    Key(sc_cli::KeySubcommand),

    /// 📄 Build a chain specification (JSON)
    BuildSpec(sc_cli::BuildSpecCmd),

    /// 🔍 Validate blocks
    CheckBlock(sc_cli::CheckBlockCmd),

    /// 📦 Export blocks to a file
    ExportBlocks(sc_cli::ExportBlocksCmd),

    /// 📊 Export the state of a given block into a chain spec
    ExportState(sc_cli::ExportStateCmd),

    /// 📥 Import blocks from a file
    ImportBlocks(sc_cli::ImportBlocksCmd),

    /// 🗑️ Remove the whole chain database (Reset)
    PurgeChain(sc_cli::PurgeChainCmd),

    /// 🔙 Revert the chain to a previous state
    Revert(sc_cli::RevertCmd),

    /// 🚀 Sub-commands concerned with benchmarking (Speed Test)
    #[command(subcommand)]
    Benchmark(frame_benchmarking_cli::BenchmarkCmd),

    /// ℹ️ Db meta columns information
    ChainInfo(sc_cli::ChainInfoCmd),
}