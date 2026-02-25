//! # RPC Module
//!
//! Wires together both Substrate-native and Ethereum-compatible JSON-RPC endpoints
//! into a single `RpcModule` that the node exposes over WebSocket and HTTP.
//!
//! ## Exposed Namespaces
//! | Namespace        | Purpose                                                   |
//! |------------------|-----------------------------------------------------------|
//! | `system_*`       | Account nonce, node health, peer count                    |
//! | `payment_*`      | Fee estimation for Substrate extrinsics                   |
//! | `eth_*`          | Full Ethereum JSON-RPC (MetaMask, ethers.js compatible)   |
//! | `eth_getFilter*` | Log/event filter management                               |
//! | `net_*`          | Network ID and peer count (Ethereum convention)           |
//! | `web3_*`         | `web3_clientVersion` and `web3_sha3`                      |
//! | `eth_subscribe`  | WebSocket subscription for new blocks and logs            |

use std::collections::BTreeMap;
use std::sync::Arc;

use sp_core::H256;
use jsonrpsee::RpcModule;

use sc_transaction_pool::ChainApi;
use sc_network_sync::SyncingService;

use fc_rpc::{
    Eth, EthFilter, EthPubSub, Net, Web3,
    EthBlockDataCacheTask,
    StorageOverride,
};
use fc_rpc_core::types::{FeeHistoryCache, FilterPool};
use fc_db::kv::Backend as FrontierBackend;
use fc_mapping_sync::{EthereumBlockNotification, EthereumBlockNotificationSinks};

use xnet_runtime::{opaque::Block, RuntimeApi};
use xnet_runtime::TransactionConverter;

use sc_rpc_api::DenyUnsafe;

// =============================================================================
// Type Aliases
// =============================================================================

/// Concrete client type — WASM executor backed by host I/O functions.
type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    sc_executor::WasmExecutor<sp_io::SubstrateHostFunctions>,
>;

/// Substrate block storage backend.
type FullBackend = sc_service::TFullBackend<Block>;

/// Frontier's key-value Ethereum database, keyed by the full client type.
type FullFrontierBackend = FrontierBackend<Block, FullClient>;

// =============================================================================
// RPC Dependencies
// =============================================================================

/// Aggregates every dependency required to build the full RPC handler.
///
/// Passed into `create_full` by the node service. Using a single struct keeps
/// the function signature readable as the number of dependencies grows.
pub struct FullDeps<C, P, A: ChainApi> {
    /// The Substrate client — provides chain state and block access.
    pub client: Arc<C>,
    /// Transaction pool — used for submitting and querying pending transactions.
    pub pool: Arc<P>,
    /// Controls which unsafe RPC methods are accessible (disabled on public nodes).
    pub deny_unsafe: DenyUnsafe,
    // --- EVM / Frontier ---
    /// Raw transaction pool graph — Frontier uses this for gas estimation.
    pub graph: Arc<sc_transaction_pool::Pool<A>>,
    /// Network service — required by `Net` and `EthPubSub` for peer info.
    pub network: Arc<dyn sc_network::service::traits::NetworkService>,
    /// Syncing service — lets the `eth_*` layer know whether the node is syncing.
    pub sync: Arc<SyncingService<Block>>,
    /// Whether this node is an authority (validator). Affects pending-block behaviour.
    pub is_authority: bool,
    /// Frontier's off-chain Ethereum database (maps Ethereum hashes to Substrate blocks).
    pub frontier_backend: Arc<FullFrontierBackend>,
    /// Reads raw EVM state (account code, storage) from the Substrate state trie.
    pub overrides: Arc<dyn StorageOverride<Block>>,
    /// In-memory pool of active Ethereum log filters.
    pub filter_pool: FilterPool,
    /// Maximum number of entries in the EIP-1559 fee-history cache.
    pub fee_history_limit: u64,
    /// Cached fee history used by `eth_feeHistory`.
    pub fee_history_cache: FeeHistoryCache,
    /// Executor for WebSocket subscription tasks.
    pub subscription_executor: sc_rpc::SubscriptionTaskExecutor,
    /// LRU cache for block receipts and transaction data (reduces DB reads).
    pub block_data_cache: Arc<EthBlockDataCacheTask<Block>>,
}

// =============================================================================
// Consensus Data Provider (Stub)
// =============================================================================

/// Minimal `ConsensusDataProvider` stub used when constructing pending blocks.
///
/// BABE pre-runtime digests are not strictly required for pending-block simulation,
/// so this returns an empty digest rather than synthesising a fake BABE proof.
pub struct DummyConsensusDataProvider;

impl fc_rpc::pending::ConsensusDataProvider<Block> for DummyConsensusDataProvider {
    fn create_digest(
        &self,
        _parent: &xnet_runtime::Header,
        _inherents: &sp_inherents::InherentData,
    ) -> Result<sp_runtime::Digest, sp_inherents::Error> {
        Ok(sp_runtime::Digest::default())
    }
}

// =============================================================================
// RPC Handler Constructor
// =============================================================================

/// Assembles the complete RPC handler for a full XNET node.
///
/// Registers Substrate-native endpoints followed by the full Frontier Ethereum
/// stack. The returned `RpcModule` is handed to `sc_service::spawn_tasks`.
pub fn create_full<A>(
    deps: FullDeps<FullClient, sc_transaction_pool::FullPool<Block, FullClient>, A>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    A: ChainApi<Block = Block> + 'static,
{
    use substrate_frame_rpc_system::{System, SystemApiServer};
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use fc_rpc::{
        EthApiServer, EthFilterApiServer, NetApiServer, Web3ApiServer, EthPubSubApiServer,
    };

    let FullDeps {
        client,
        pool,
        deny_unsafe,
        graph,
        network,
        sync,
        is_authority,
        frontier_backend,
        overrides,
        filter_pool,
        fee_history_limit,
        fee_history_cache,
        subscription_executor,
        block_data_cache,
    } = deps;

    let mut module = RpcModule::new(());

    // --- Substrate standard endpoints ---
    module.merge(System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc())?;
    module.merge(TransactionPayment::new(client.clone()).into_rpc())?;

    // --- Ethereum RPC (MetaMask / ethers.js entry point) ---
    //
    // Eth::new argument order (Frontier stable2407 — 16 parameters):
    //   1.  client
    //   2.  pool
    //   3.  graph
    //   4.  convert_transaction: Option<CT>
    //   5.  sync
    //   6.  signers: Vec<Box<dyn EthSigner>>
    //   7.  storage_override
    //   8.  backend
    //   9.  is_authority
    //   10. block_data_cache
    //   11. fee_history_cache
    //   12. fee_history_cache_limit: u64
    //   13. execute_gas_limit_multiplier: u64
    //   14. forced_parent_hashes: Option<BTreeMap<H256, H256>>
    //   15. pending_create_inherent_data_providers (async closure)
    //   16. pending_consensus_data_provider: Option<Box<dyn ConsensusDataProvider<B>>>
    let pubsub_notification_sinks: Arc<EthereumBlockNotificationSinks<EthereumBlockNotification<Block>>> =
        Arc::new(parking_lot::Mutex::new(Vec::new()));

    module.merge(
        Eth::<Block, FullClient, _, TransactionConverter, FullBackend, A, _, ()>::new(
            client.clone(),
            pool.clone(),
            graph.clone(),
            Some(TransactionConverter),
            sync.clone(),
            vec![],                                   // No custom signers
            overrides.clone(),
            frontier_backend.clone(),
            is_authority,
            block_data_cache.clone(),
            fee_history_cache.clone(),
            fee_history_limit,
            10u64,                                    // Gas-limit multiplier for estimates
            None::<BTreeMap<H256, H256>>,             // No forced parent hash overrides
            move |_, _| async move {                  // Pending-block inherent provider (no-op)
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            },
            Some(Box::new(DummyConsensusDataProvider)),
        )
        .into_rpc(),
    )?;

    // --- Ethereum log filter endpoints (`eth_newFilter`, `eth_getFilterLogs`, …) ---
    module.merge(
        EthFilter::new(
            client.clone(),
            frontier_backend.clone(),
            graph.clone(),
            filter_pool.clone(),
            500usize,                 // Maximum number of concurrent stored filters
            fee_history_limit as u32, // Reuse the fee-history window as the log lookback cap
            block_data_cache.clone(),
        )
        .into_rpc(),
    )?;

    // --- `net_*` endpoints (peer count, version, listening status) ---
    module.merge(
        Net::new(
            client.clone(),
            network.clone(),
            true, // Report peer count as a hex string (Ethereum convention)
        )
        .into_rpc(),
    )?;

    // --- `web3_clientVersion` and `web3_sha3` ---
    module.merge(Web3::new(client.clone()).into_rpc())?;

    // --- `eth_subscribe` WebSocket subscriptions (new headers, logs, pending txs) ---
    module.merge(
        EthPubSub::new(
            pool.clone(),
            client.clone(),
            sync.clone(),
            subscription_executor,
            overrides.clone(),
            pubsub_notification_sinks,
        )
        .into_rpc(),
    )?;

    Ok(module)
}
