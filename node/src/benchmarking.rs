//! # Benchmarking Helpers
//!
//! Provides signed extrinsic builders and inherent data factories used by the
//! Substrate benchmarking framework when measuring block and extrinsic weights.
//!
//! All items in this module are gated behind `#[cfg(feature = "runtime-benchmarks")]`
//! so the compiler does not emit dead-code warnings on normal (non-benchmark) builds.

// Only compile this module when the runtime-benchmarks feature is active.
// This suppresses all dead_code / unused warnings on standard builds.
#![cfg(feature = "runtime-benchmarks")]

use crate::service::FullClient;
use runtime::{AccountId, Balance, BalancesCall, SystemCall};
use xnet_runtime as runtime;

use sc_cli::Result;
use sc_client_api::BlockBackend;
use sp_core::{Encode, Pair};
use sp_inherents::{InherentData, InherentDataProvider};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{OpaqueExtrinsic, SaturatedConversion};

use std::{sync::Arc, time::Duration};

// =============================================================================
// Remark Builder
// =============================================================================

/// Builds `System::remark` extrinsics for block-overhead benchmarking.
///
/// `remark` is the cheapest possible call — it only stores a byte vector
/// in the extrinsic body. This makes it ideal for isolating per-block overhead
/// from per-extrinsic execution costs.
pub struct RemarkBuilder {
	client: Arc<FullClient>,
}

impl RemarkBuilder {
	/// Creates a new `RemarkBuilder` backed by the given client.
	pub fn new(client: Arc<FullClient>) -> Self {
		Self { client }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for RemarkBuilder {
	fn pallet(&self) -> &str {
		"system"
	}

	fn extrinsic(&self) -> &str {
		"remark"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			SystemCall::remark { remark: vec![] }.into(),
			nonce,
		)
		.into();
		Ok(extrinsic)
	}
}

// =============================================================================
// Transfer Keep-Alive Builder
// =============================================================================

/// Builds `Balances::transfer_keep_alive` extrinsics for extrinsic-weight benchmarking.
///
/// Unlike `transfer`, `transfer_keep_alive` reverts if the sender's balance would
/// fall below the existential deposit, making it a more production-representative call.
pub struct TransferKeepAliveBuilder {
	client: Arc<FullClient>,
	dest: AccountId,
	value: Balance,
}

impl TransferKeepAliveBuilder {
	/// Creates a new builder that will transfer `value` tokens to `dest`.
	pub fn new(client: Arc<FullClient>, dest: AccountId, value: Balance) -> Self {
		Self { client, dest, value }
	}
}

impl frame_benchmarking_cli::ExtrinsicBuilder for TransferKeepAliveBuilder {
	fn pallet(&self) -> &str {
		"balances"
	}

	fn extrinsic(&self) -> &str {
		"transfer_keep_alive"
	}

	fn build(&self, nonce: u32) -> std::result::Result<OpaqueExtrinsic, &'static str> {
		let acc = Sr25519Keyring::Bob.pair();
		let extrinsic: OpaqueExtrinsic = create_benchmark_extrinsic(
			self.client.as_ref(),
			acc,
			BalancesCall::transfer_keep_alive { dest: self.dest.clone().into(), value: self.value }
				.into(),
			nonce,
		)
		.into();
		Ok(extrinsic)
	}
}

// =============================================================================
// Extrinsic Factory
// =============================================================================

/// Constructs a fully signed `UncheckedExtrinsic` suitable for benchmarking.
///
/// The `SignedExtra` tuple must match the definition in `runtime/src/lib.rs`
/// exactly — any mismatch will cause signature verification to fail at runtime.
pub fn create_benchmark_extrinsic(
	client: &FullClient,
	sender: sp_core::sr25519::Pair,
	call: runtime::RuntimeCall,
	nonce: u32,
) -> runtime::UncheckedExtrinsic {
	let genesis_hash = client.block_hash(0).ok().flatten().expect("Genesis block exists; qed");
	let best_hash = client.chain_info().best_hash;
	let best_block = client.chain_info().best_number;

	// Mortality window: half the block-hash count, clamped to a power of two.
	let period = runtime::BlockHashCount::get()
		.checked_next_power_of_two()
		.map(|c| c / 2)
		.unwrap_or(2) as u64;

	let extra: runtime::SignedExtra = (
		frame_system::CheckNonZeroSender::<runtime::Runtime>::new(),
		frame_system::CheckSpecVersion::<runtime::Runtime>::new(),
		frame_system::CheckTxVersion::<runtime::Runtime>::new(),
		frame_system::CheckGenesis::<runtime::Runtime>::new(),
		frame_system::CheckEra::<runtime::Runtime>::from(sp_runtime::generic::Era::mortal(
			period,
			best_block.saturated_into(),
		)),
		frame_system::CheckNonce::<runtime::Runtime>::from(nonce),
		frame_system::CheckWeight::<runtime::Runtime>::new(),
		// Zero tip — benchmarks should not factor in priority fees.
		pallet_transaction_payment::ChargeTransactionPayment::<runtime::Runtime>::from(0),
	);

	let raw_payload = runtime::SignedPayload::from_raw(
		call.clone(),
		extra.clone(),
		(
			(),
			runtime::VERSION.spec_version,
			runtime::VERSION.transaction_version,
			genesis_hash,
			best_hash,
			(),
			(),
			(),
		),
	);

	let signature = raw_payload.using_encoded(|e| sender.sign(e));

	runtime::UncheckedExtrinsic::new_signed(
		call,
		sp_runtime::AccountId32::from(sender.public()).into(),
		runtime::Signature::Sr25519(signature),
		extra,
	)
}

// =============================================================================
// Inherent Data
// =============================================================================

/// Creates a minimal `InherentData` set for benchmark block execution.
///
/// Only the timestamp inherent is provided (set to genesis time, i.e. zero),
/// since benchmarks do not require a real clock reading.
pub fn inherent_benchmark_data() -> Result<InherentData> {
	let mut inherent_data = InherentData::new();
	let d = Duration::from_millis(0);
	let timestamp = sp_timestamp::InherentDataProvider::new(d.into());

	futures::executor::block_on(timestamp.provide_inherent_data(&mut inherent_data))
		.map_err(|e| format!("Failed to provide timestamp inherent: {:?}", e))?;

	Ok(inherent_data)
}
