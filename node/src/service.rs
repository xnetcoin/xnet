//! # Node Service
//!
//! Bootstraps and runs the full XNET node: client, networking, consensus workers,
//! Frontier EVM sync tasks, and the RPC server. Two public entry points are exposed:
//!
//! - [`new_partial`] — initialises the components that are shared between all node
//!   modes (full node, light client, benchmarking). Returns a `PartialComponents`
//!   bundle that other commands can destructure without starting the whole network.
//!
//! - [`new_full`] — calls `new_partial`, then brings up networking, the BABE
//!   block-authoring worker, the GRANDPA finality voter, Frontier EVM background
//!   tasks, and the JSON-RPC server.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::{FutureExt, StreamExt};
use sc_client_api::{Backend, BlockBackend, BlockchainEvents};
use sc_consensus_babe::SlotProportion;
use sc_network::config::FullNetworkConfiguration;
use sc_network::Litep2pNetworkBackend;
use sc_network_sync::WarpSyncConfig;
use sc_service::{error::Error as ServiceError, Configuration, TaskManager};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool_api::OffchainTransactionPoolFactory;

use sc_executor::WasmExecutor;
use xnet_runtime::{self, opaque::Block, RuntimeApi};

// Frontier (Ethereum compatibility layer) imports.
use fc_db::kv::Backend as FrontierBackend;
use fc_mapping_sync::SyncStrategy;
use fc_rpc::{EthBlockDataCacheTask, EthTask, StorageOverrideHandler};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};

// =============================================================================
// Type Aliases
// =============================================================================

/// Substrate full client with WASM executor.
pub type FullClient =
	sc_service::TFullClient<Block, RuntimeApi, WasmExecutor<sp_io::SubstrateHostFunctions>>;

/// Substrate RocksDB backend.
pub type FullBackend = sc_service::TFullBackend<Block>;

/// Longest-chain fork-choice rule (standard for BABE/GRANDPA chains).
pub type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;

/// GRANDPA block-import wrapper (adds justification logic around the inner importer).
pub type GrandpaBlockImport =
	sc_consensus_grandpa::GrandpaBlockImport<FullBackend, Block, FullClient, FullSelectChain>;

/// BABE block-import wrapper (validates VRF proofs before passing to GRANDPA).
pub type BabeBlockImport =
	sc_consensus_babe::BabeBlockImport<Block, FullClient, GrandpaBlockImport>;

/// Standard full transaction pool (validates and prioritises Substrate extrinsics).
pub type TransactionPool = sc_transaction_pool::FullPool<Block, FullClient>;

/// Frontier's key-value Ethereum mapping database.
pub type FullFrontierBackend = FrontierBackend<Block, FullClient>;

// =============================================================================
// Constants
// =============================================================================

/// Number of blocks retained in the EIP-1559 fee-history cache.
pub const FEE_HISTORY_LIMIT: u64 = 2048;

/// Number of blocks after which expired Ethereum log filters are evicted.
pub const FILTER_RETAIN_THRESHOLD: u64 = 100;

// =============================================================================
// Frontier Database Helpers
// =============================================================================

/// Returns the filesystem path for a Frontier sub-database (e.g. `"db"` or `"meta"`).
pub fn frontier_database_dir(config: &Configuration, path: &str) -> PathBuf {
	config.base_path.config_dir(config.chain_spec.id()).join("frontier").join(path)
}

/// Opens (or creates) the Frontier RocksDB backend that maps Ethereum block hashes
/// to their Substrate equivalents and stores transaction receipts.
///
/// The `DatabaseSource::RocksDb` variant accepts the client directly rather than a
/// settings struct — this matches the Frontier stable2407 API.
#[allow(clippy::result_large_err)]
pub fn open_frontier_backend(
	client: Arc<FullClient>,
	config: &Configuration,
) -> Result<Arc<FullFrontierBackend>, ServiceError> {
	let db_path = frontier_database_dir(config, "db");
	FrontierBackend::open(
		Arc::clone(&client),
		&fc_db::DatabaseSource::RocksDb { path: db_path.clone(), cache_size: 0 },
		&db_path,
	)
	.map(Arc::new)
	.map_err(|e| ServiceError::Application(format!("Frontier backend open error: {e:?}").into()))
}

// =============================================================================
// Partial Components
// =============================================================================

/// Initialises the core node components without starting the network or consensus.
///
/// Returns a `PartialComponents` bundle containing the client, backend, import
/// queue, transaction pool, and a tuple of additional items needed by `new_full`.
/// This function is also called by benchmarking and maintenance sub-commands that
/// do not need a live P2P network.
#[allow(clippy::result_large_err, clippy::type_complexity)]
pub fn new_partial(
	config: &Configuration,
) -> Result<
	sc_service::PartialComponents<
		FullClient,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		TransactionPool,
		(
			BabeBlockImport,
			sc_consensus_grandpa::LinkHalf<Block, FullClient, FullSelectChain>,
			sc_consensus_babe::BabeLink<Block>,
			Option<Telemetry>,
			sc_consensus_babe::BabeWorkerHandle<Block>,
			Arc<FullFrontierBackend>,
			FilterPool,
			FeeHistoryCache,
		),
	>,
	ServiceError,
> {
	// Set up telemetry — if no endpoints are configured this is a no-op.
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let executor = sc_service::new_wasm_executor::<sp_io::SubstrateHostFunctions>(&config.executor);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, t)| t.handle()),
			executor,
		)?;

	let client: Arc<FullClient> = Arc::new(client);
	let backend: Arc<FullBackend> = backend;

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool: Arc<TransactionPool> = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);

	// GRANDPA wraps every imported block to accumulate justifications.
	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		0u32,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|t: &Telemetry| t.handle()),
	)?;

	// BABE wraps GRANDPA to add VRF proof verification.
	let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		grandpa_block_import.clone(),
		client.clone(),
	)?;

	let slot_duration = babe_link.config().slot_duration();

	// The import queue validates and queues blocks received from peers.
	let (import_queue, babe_worker_handle) = sc_consensus_babe::import_queue(
		sc_consensus_babe::ImportQueueParams {
			link: babe_link.clone(),
			block_import: babe_block_import.clone(),
			justification_import: Some(Box::new(grandpa_block_import.clone())),
			client: client.clone(),
			select_chain: select_chain.clone(),
			create_inherent_data_providers: move |_, ()| async move {
				// Provide timestamp and BABE slot inherents for every imported block.
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
				let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp, slot_duration,
                    );
				Ok((slot, timestamp))
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			telemetry: telemetry.as_ref().map(|t: &Telemetry| t.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
		},
	)?;

	let frontier_backend = open_frontier_backend(client.clone(), config)?;

	// In-memory filter pool for `eth_newFilter` / `eth_getLogs`.
	let filter_pool: FilterPool = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
	// Rolling cache of base-fee history used by `eth_feeHistory`.
	let fee_history_cache: FeeHistoryCache = Arc::new(std::sync::Mutex::new(BTreeMap::new()));

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (
			babe_block_import,
			grandpa_link,
			babe_link,
			telemetry,
			babe_worker_handle,
			frontier_backend,
			filter_pool,
			fee_history_cache,
		),
	})
}

// =============================================================================
// Full Node
// =============================================================================

/// Starts a fully operational XNET node: network, consensus, EVM sync, and RPC.
///
/// Performs the following in order:
/// 1. Calls `new_partial` to obtain client, pools, and import queue.
/// 2. Builds the libp2p / litep2p network and GRANDPA gossip protocol.
/// 3. Spawns the Frontier EVM mapping sync worker and filter/fee-history tasks.
/// 4. Registers the JSON-RPC handler (Substrate + Ethereum endpoints).
/// 5. Starts the BABE block-production loop (authority nodes only).
/// 6. Starts the GRANDPA finality voter.
#[allow(clippy::result_large_err)]
pub fn new_full(config: Configuration) -> Result<TaskManager, ServiceError> {
	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other:
			(
				babe_block_import,
				grandpa_link,
				babe_link,
				telemetry,
				babe_worker_handle,
				frontier_backend,
				filter_pool,
				fee_history_cache,
			),
	} = new_partial(&config)?;

	// Annotate types explicitly to help the compiler with inference across modules.
	let client: Arc<FullClient> = client;
	let backend: Arc<FullBackend> = backend;
	let mut telemetry: Option<Telemetry> = telemetry;
	let frontier_backend: Arc<FullFrontierBackend> = frontier_backend;
	let filter_pool: FilterPool = filter_pool;
	let fee_history_cache: FeeHistoryCache = fee_history_cache;
	let transaction_pool: Arc<TransactionPool> = transaction_pool;

	let metrics = sc_network::config::NotificationMetrics::new(config.prometheus_registry());

	let mut net_config: FullNetworkConfiguration<
		Block,
		<Block as sp_runtime::traits::Block>::Hash,
		Litep2pNetworkBackend,
	> = FullNetworkConfiguration::new(&config.network, config.prometheus_registry().cloned());

	let genesis_hash: <Block as sp_runtime::traits::Block>::Hash =
		client.block_hash(0u32).ok().flatten().expect("Genesis block exists; qed");

	// Register the GRANDPA gossip sub-protocol so peers can exchange votes and justifications.
	let grandpa_protocol_name =
		sc_consensus_grandpa::protocol_standard_name(&genesis_hash, &config.chain_spec);

	let (grandpa_protocol_config, grandpa_notification_service) =
		sc_consensus_grandpa::grandpa_peers_set_config::<Block, Litep2pNetworkBackend>(
			grandpa_protocol_name.clone(),
			metrics.clone(),
			net_config.peer_store_handle(),
		);
	net_config.add_notification_protocol(grandpa_protocol_config);

	// GRANDPA warp-sync provider allows new nodes to fast-sync to the latest
	// finalized checkpoint rather than replaying every block from genesis.
	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		grandpa_link.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
			block_relay: None,
			metrics,
		})?;

	// Offchain workers handle tasks that do not need to be on-chain (e.g. ImOnline heartbeats).
	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: Arc::new(network.clone()),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	// =========================================================================
	// Frontier EVM Background Tasks
	// =========================================================================

	// `StorageOverrideHandler` reads EVM account/storage state from the trie.
	let overrides: Arc<dyn fc_storage::StorageOverride<Block>> =
		Arc::new(StorageOverrideHandler::<Block, _, _>::new(client.clone()));

	// LRU cache for block receipts and transaction data — reduces hot-path DB reads.
	let block_data_cache = Arc::new(EthBlockDataCacheTask::new(
		task_manager.spawn_handle(),
		overrides.clone(),
		50, // Cached block count
		50, // Cached transaction count
		config.prometheus_registry().cloned(),
	));

	// Mapping sync: watches for new Substrate blocks and writes Ethereum-format
	// receipts and hash mappings into the Frontier database.
	task_manager.spawn_essential_handle().spawn(
		"frontier-mapping-sync-worker",
		Some("frontier"),
		fc_mapping_sync::kv::MappingSyncWorker::new(
			client.import_notification_stream(),
			Duration::new(6, 0), // Retry interval — matches block time
			client.clone(),
			backend.clone(),
			overrides.clone(),
			frontier_backend.clone(),
			3, // Maximum in-flight import tasks
			0, // Delay before first sync attempt
			SyncStrategy::Normal,
			sync_service.clone(),
			Arc::new(parking_lot::Mutex::new(Vec::new())),
		)
		.for_each(|()| futures::future::ready(())),
	);

	// Filter pool janitor: evicts Ethereum log filters older than FILTER_RETAIN_THRESHOLD blocks.
	task_manager.spawn_essential_handle().spawn(
		"frontier-filter-pool",
		Some("frontier"),
		EthTask::filter_pool_task(client.clone(), filter_pool.clone(), FILTER_RETAIN_THRESHOLD),
	);

	// Fee-history updater: maintains the rolling EIP-1559 base-fee history cache.
	task_manager.spawn_essential_handle().spawn(
		"frontier-fee-history",
		Some("frontier"),
		EthTask::fee_history_task(
			client.clone(),
			overrides.clone(),
			fee_history_cache.clone(),
			FEE_HISTORY_LIMIT,
		),
	);

	// =========================================================================
	// RPC Handler
	// =========================================================================

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks: Option<()> = None; // No backoff — author every eligible slot
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();
	let is_authority = role.is_authority();

	let rpc_extensions_builder = {
		let client = client.clone();
		let pool = transaction_pool.clone();
		let network = network.clone();
		let sync_service = sync_service.clone();
		let frontier_backend = frontier_backend.clone();
		let filter_pool = filter_pool.clone();
		let fee_history_cache = fee_history_cache.clone();
		let block_data_cache = block_data_cache.clone();
		let overrides = overrides.clone();

		Box::new(move |subscription_executor: sc_rpc::SubscriptionTaskExecutor| {
			let deps = crate::rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				graph: pool.pool().clone(),
				network: network.clone(),
				sync: sync_service.clone(),
				is_authority,
				frontier_backend: frontier_backend.clone(),
				overrides: overrides.clone(),
				filter_pool: filter_pool.clone(),
				fee_history_limit: FEE_HISTORY_LIMIT,
				fee_history_cache: fee_history_cache.clone(),
				subscription_executor,
				block_data_cache: block_data_cache.clone(),
			};

			crate::rpc::create_full(deps).map_err(|e| sc_service::Error::Application(e))
		})
	};

	let _rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		client: client.clone(),
		backend: backend.clone(),
		task_manager: &mut task_manager,
		keystore: keystore_container.keystore(),
		transaction_pool: transaction_pool.clone(),
		rpc_builder: rpc_extensions_builder,
		network: network.clone(),
		system_rpc_tx,
		tx_handler_controller,
		sync_service: sync_service.clone(),
		telemetry: telemetry.as_mut(),
	})?;

	// =========================================================================
	// BABE Block Authoring (Validators Only)
	// =========================================================================

	if role.is_authority() {
		let proposer_factory = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|t: &Telemetry| t.handle()),
		);

		let slot_duration = babe_link.config().slot_duration();

		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore_container.keystore(),
			client: client.clone(),
			select_chain,
			env: proposer_factory,
			block_import: babe_block_import,
			sync_oracle: sync_service.clone(),
			justification_sync_link: sync_service.clone(),
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
				let slot =
                    sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                        *timestamp, slot_duration,
                    );
				Ok((slot, timestamp))
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			// Claim a slot if we produced the last 2/3 of the previous slot's time window.
			block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|t: &Telemetry| t.handle()),
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking(
			"babe-proposer",
			Some("block-authoring"),
			babe,
		);

		// Keep the BABE worker handle alive for the lifetime of the node.
		task_manager.spawn_handle().spawn(
			"babe-worker-handle-keeper",
			None,
			async move {
				let _handle = babe_worker_handle;
				std::future::pending::<()>().await
			}
			.boxed(),
		);
	}

	// =========================================================================
	// GRANDPA Finality Voter
	// =========================================================================

	if enable_grandpa {
		let grandpa_config = sc_consensus_grandpa::Config {
			// How often GRANDPA gossips pre-vote and pre-commit messages.
			gossip_duration: Duration::from_millis(333),
			// Generate a justification every 512 blocks for warp-sync support.
			justification_generation_period: 512,
			name: Some(name),
			// Observer mode runs GRANDPA without signing — useful for archive nodes.
			observer_enabled: false,
			keystore: if role.is_authority() { Some(keystore_container.keystore()) } else { None },
			local_role: role,
			telemetry: telemetry.as_ref().map(|t: &Telemetry| t.handle()),
			protocol_name: grandpa_protocol_name,
		};

		let grandpa_params = sc_consensus_grandpa::GrandpaParams {
			config: grandpa_config,
			link: grandpa_link,
			network: network.clone(),
			sync: sync_service.clone(),
			notification_service: grandpa_notification_service,
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state: sc_consensus_grandpa::SharedVoterState::empty(),
			telemetry: telemetry.as_ref().map(|t: &Telemetry| t.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
		};

		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_params)?,
		);
	}

	// Start the network — after this point the node accepts peer connections.
	network_starter.start_network();
	Ok(task_manager)
}
