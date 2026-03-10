//! # Command Dispatcher
//!
//! Maps CLI subcommands to their corresponding Substrate runner functions.
//! Each arm initialises the minimum set of node components needed for the
//! requested operation (e.g. block import only needs the import queue, not
//! the full networking stack), keeping startup time short.

use crate::{
	chain_spec,
	cli::{Cli, Subcommand},
	service,
};
use sc_cli::SubstrateCli;
use sc_service::PartialComponents;
use xnet_runtime::Block;

/// Binds the `Cli` type to the Substrate CLI framework so that it can load
/// chain specs, report the node name, and surface version information.
impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"XNET Protocol Node".into()
	}

	fn impl_version() -> String {
		env!("CARGO_PKG_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://xnetcoin.org".into()
	}

	fn copyright_start_year() -> i32 {
		2024
	}

	/// Resolves a chain ID string to a boxed `ChainSpec`.
	///
	/// Recognised IDs:
	/// - `"dev"` / `"xnet_dev"` — single-node development chain
	/// - `"testnet"` / `"xnet_testnet"` — public testnet
	/// - `"mainnet"` / `"xnet"` / `""` — production mainnet
	/// - anything else — treated as a path to a JSON chain-spec file
	fn load_spec(&self, id: &str) -> Result<Box<dyn sc_cli::ChainSpec>, String> {
		Ok(match id {
			"dev" | "xnet_dev" => Box::new(chain_spec::development_config()?),
			"testnet" | "xnet_testnet" => Box::new(chain_spec::testnet_config()?),
			"xnet" | "xnet_mainnet" | "" => Box::new(chain_spec::mainnet_config()?),
			path => {
				Box::new(chain_spec::ChainSpec::from_json_file(std::path::PathBuf::from(path))?)
			},
		})
	}
}

/// Parses command-line arguments and dispatches to the appropriate handler.
///
/// Returns `Ok(())` on clean exit, or a `sc_cli::Error` if the operation
/// failed or the arguments were invalid.
#[allow(clippy::result_large_err)]
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(Subcommand::Key(cmd)) => cmd.run(&cli),

		Some(Subcommand::BuildSpec(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
		},

		Some(Subcommand::CheckBlock(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::ExportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.database), task_manager))
			})
		},

		Some(Subcommand::ExportState(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, .. } = service::new_partial(&config)?;
				Ok((cmd.run(client, config.chain_spec), task_manager))
			})
		},

		Some(Subcommand::ImportBlocks(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, import_queue, .. } =
					service::new_partial(&config)?;
				Ok((cmd.run(client, import_queue), task_manager))
			})
		},

		Some(Subcommand::PurgeChain(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run(config.database))
		},

		Some(Subcommand::Revert(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { client, task_manager, backend, .. } =
					service::new_partial(&config)?;
				// Also revert GRANDPA justifications stored in the aux DB.
				let aux_revert = Box::new(move |client, _, blocks| {
					sc_consensus_grandpa::revert(client, blocks)?;
					Ok(())
				});
				Ok((cmd.run(client, backend, Some(aux_revert)), task_manager))
			})
		},

		Some(Subcommand::Benchmark(cmd)) => {
			let runner = cli.create_runner(cmd)?;

			runner.sync_run(|config| {
				// Pallet-level weight benchmarking — measures per-extrinsic execution time.
				if let frame_benchmarking_cli::BenchmarkCmd::Pallet(pallet_cmd) = cmd {
					pallet_cmd.run_with_spec::<sp_runtime::traits::HashingFor<Block>, ()>(Some(
						config.chain_spec,
					))
				}
				// Block-level overhead benchmarking — measures block initialisation cost.
				else if let frame_benchmarking_cli::BenchmarkCmd::Block(block_cmd) = cmd {
					let PartialComponents { client, .. } = service::new_partial(&config)?;
					block_cmd.run(client)
				} else {
					Err("Unsupported benchmark subcommand. Use 'pallet' or 'block'.".into())
				}
			})
		},

		Some(Subcommand::ChainInfo(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.sync_run(|config| cmd.run::<Block>(&config))
		},

		// No subcommand: start the full node.
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node_until_exit(|config| async move {
				service::new_full(config).map_err(sc_cli::Error::Service)
			})
		},
	}
}
