//! A collection of node-specific RPC methods.
//! XNET Protocol uses the `sc-rpc` crate, which defines the core RPC layer.

#![warn(missing_docs)]

use std::sync::Arc;

use jsonrpsee::RpcModule;
use xnet_runtime::{opaque::Block, AccountId, Balance, Nonce};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};

pub use sc_rpc_api::DenyUnsafe;

/// Full client dependencies.
pub struct FullDeps<C, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
}

/// Instantiate all full RPC extensions.
pub fn create_full<C, P>(
    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
    C: Send + Sync + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
{
    use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApiServer};
    use substrate_frame_rpc_system::{System, SystemApiServer};

    let FullDeps { client, pool, deny_unsafe } = deps;

    let mut module = RpcModule::new(());

    // System RPC
    let system_rpc = System::new(client.clone(), pool.clone(), deny_unsafe).into_rpc();
    module.merge(system_rpc)?;

    // TransactionPayment RPC
    let tx_payment_rpc = TransactionPayment::new(client).into_rpc();
    module.merge(tx_payment_rpc)?;

    Ok(module)
}
