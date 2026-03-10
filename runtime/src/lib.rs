//! # XNET Runtime
//!
//! Core runtime configuration for the XNET blockchain node. This crate wires together
//! all FRAME pallets — consensus (BABE + GRANDPA), staking, treasury, EVM, and WASM
//! smart contracts — into a single cohesive runtime binary that compiles to both native
//! and WebAssembly targets.
//!
//! ## Architecture
//!
//! - **Consensus**: BABE block production + GRANDPA finality
//! - **Economy**: NPoS staking, on-chain treasury, block rewards with halving
//! - **Smart Contracts**: Dual-stack — EVM (Solidity) and ink! (WASM)
//! - **Governance**: Sudo key for bootstrapping; extendable to on-chain governance

#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use codec::Decode;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_api::impl_runtime_apis;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{Encode, Get, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{
		AccountIdLookup, BlakeTwo256, Block as BlockT, Convert, Dispatchable, IdentifyAccount,
		NumberFor, Verify,
	},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, KeyTypeId, MultiSignature, Perbill, Permill,
};
use sp_std::prelude::*;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;

use frame_election_provider_support::{onchain, SequentialPhragmen};
use frame_support::{
	construct_runtime, parameter_types,
	traits::{
		fungible::Credit, ConstU128, ConstU32, ConstU64, ConstU8, FindAuthor, KeyOwnerProofSystem,
	},
	weights::{
		constants::{RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND},
		Weight,
	},
	PalletId,
};
use frame_system::EnsureRoot;
use sp_staking::currency_to_vote::U128CurrencyToVote;

// EVM imports
use fp_rpc::TransactionStatus;
use frame_support::traits::OnFinalize;
use pallet_ethereum::{Call::transact, Transaction as EthereumTransaction};
use pallet_evm::{
	Account as EVMAccount, EnsureAddressTruncated, FeeCalculator, HashedAddressMapping, Runner,
};
use sp_core::{H160, H256, U256};
use sp_runtime::ConsensusEngineId;

// EVM precompile set — standard Ethereum precompiles 0x01–0x09.
mod precompiles;
use precompiles::XnetPrecompiles;

use pallet_zk_verifier;

pub use frame_system::Call as SystemCall;
pub use pallet_balances::Call as BalancesCall;
pub use pallet_timestamp::Call as TimestampCall;

// =============================================================================
// Primitive Type Aliases
// =============================================================================

/// Block number type. A `u32` supports ~136 years of blocks at one block per second.
pub type BlockNumber = u32;
/// Signature type — supports Sr25519, Ed25519, and ECDSA via `MultiSignature`.
pub type Signature = MultiSignature;
/// Account ID derived from the signature's signer public key.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
/// Native token balance, stored as a `u128` to support 18-decimal fixed-point arithmetic.
pub type Balance = u128;
/// Account nonce (transaction counter).
pub type Nonce = u32;
/// 32-byte Blake2 hash used throughout the runtime.
pub type Hash = sp_core::H256;

// =============================================================================
// Opaque Types (used by the node side, not exposed in metadata)
// =============================================================================

pub mod opaque {
	use super::*;
	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub babe: BabeId,
			pub grandpa: GrandpaId,
			pub im_online: ImOnlineId,
		}
	}
}

// =============================================================================
// Runtime Version
// =============================================================================

/// Identifies this runtime to the outside world. Bump `spec_version` on every
/// storage-breaking upgrade so nodes can detect incompatible chain states.
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("xnetcoin"),
	impl_name: create_runtime_str!("xnetcoin"),
	authoring_version: 1,
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
	pub BlockLength: frame_system::limits::BlockLength =
		frame_system::limits::BlockLength::max_with_normal_ratio(
			5 * 1024 * 1024,
			NORMAL_DISPATCH_RATIO,
		);
	pub const SS58Prefix: u16 = 888;
}

// =============================================================================
// Tokenomics & Time Constants
// =============================================================================

/// Target block time in milliseconds (6 seconds).
pub const MILLISECS_PER_BLOCK: u64 = 6000;
/// BABE slot duration — aligned 1:1 with block time.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;
/// Approximate number of blocks produced in one minute.
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
/// Approximate blocks per hour.
pub const HOURS: BlockNumber = MINUTES * 60;
/// Approximate blocks per day.
pub const DAYS: BlockNumber = HOURS * 24;

/// One XNET token — 18 decimal places, stored as `u128`.
pub const UNIT: Balance = 1_000_000_000_000_000_000;
/// 0.001 XNET.
pub const MILLIUNIT: Balance = UNIT / 1_000;
/// 0.000001 XNET.
pub const MICROUNIT: Balance = MILLIUNIT / 1_000;

/// Tokens emitted per block. Halves at every `HalvingInterval` blocks,
/// mirroring Bitcoin's supply schedule.
pub const BLOCK_REWARD: Balance = 1_117 * MILLIUNIT;
/// Minimum self-bond required to register as a validator (8,000 XNET).
pub const MIN_VALIDATOR_BOND: Balance = 8_000 * UNIT;
/// Minimum bond required to nominate a validator (1,000 XNET).
pub const MIN_NOMINATOR_BOND: Balance = 1_000 * UNIT;
/// Minimum account balance before the account is reaped from storage.
pub const EXISTENTIAL_DEPOSIT: Balance = MILLIUNIT;

/// Hard cap on total XNET token supply: **53,000,000 XNET**.
///
/// Derivation:
/// - Genesis premine:  6,000,000 XNET  (founder allocation)
/// - Block rewards:   ~47,000,000 XNET  (sum of all halving periods)
///   = BLOCK_REWARD × HalvingInterval × 2  (geometric series)
///   = 1.117 XNET    × 21,038,400       
/// - Total:           ~53,000,000 XNET
///
/// The block-reward pallet enforces this cap at the minting step; once total
/// issuance reaches `MAX_SUPPLY` no further tokens are ever created.
pub const MAX_SUPPLY: Balance = 53_000_000 * UNIT;

// =============================================================================
// Pallet Configurations
// =============================================================================

// --- System ---

impl frame_system::Config for Runtime {
	// TESTNET: Allow all calls for testing. Before mainnet, integrate
	// pallet-tx-pause here so individual pallets can be frozen in emergencies.
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = BlockWeights;
	type BlockLength = BlockLength;
	type AccountId = AccountId;
	type RuntimeCall = RuntimeCall;
	type Lookup = AccountIdLookup<AccountId, ()>;
	type Nonce = Nonce;
	type Hash = Hash;
	type Hashing = BlakeTwo256;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = Version;
	type PalletInfo = PalletInfo;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type AccountData = pallet_balances::AccountData<Balance>;
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Block = Block;
	type RuntimeTask = RuntimeTask;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

// --- BABE (Block Authorship By Expected) ---

parameter_types! {
	pub const EpochDuration: u64 = HOURS as u64;
	pub const ExpectedBlockTime: u64 = MILLISECS_PER_BLOCK;
	pub const ReportLongevity: u64 =
		BondingDuration::get() as u64 * SessionsPerEra::get() as u64 * EpochDuration::get();
}

impl pallet_babe::Config for Runtime {
	type EpochDuration = EpochDuration;
	type ExpectedBlockTime = ExpectedBlockTime;
	type EpochChangeTrigger = pallet_babe::ExternalTrigger;
	type DisabledValidators = Session;
	type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, BabeId)>>::Proof;
	type EquivocationReportSystem =
		pallet_babe::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
	type WeightInfo = ();
	type MaxAuthorities = ConstU32<1000>;
	type MaxNominators = ConstU32<1000>;
}

// --- GRANDPA (Byzantine-fault-tolerant finality) ---

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	type MaxAuthorities = ConstU32<1000>;
	/// Retain GRANDPA authority set history for `BondingDuration × SessionsPerEra` entries
	/// (28 × 24 = 672). This allows light clients and bridges to verify finality proofs
	/// for any block within the last unbonding period.
	type MaxSetIdSessionEntries = ConstU64<672>;
	type KeyOwnerProof = <Historical as KeyOwnerProofSystem<(KeyTypeId, GrandpaId)>>::Proof;
	type EquivocationReportSystem =
		pallet_grandpa::EquivocationReportSystem<Self, Offences, Historical, ReportLongevity>;
	type MaxNominators = ConstU32<10000>;
}

// --- Timestamp ---

impl pallet_timestamp::Config for Runtime {
	type Moment = u64;
	type OnTimestampSet = Babe;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}
// --- Treasury ---
// G'aznaga chang tokenlarni tushirib beruvchi maxsus ko'prik
pub struct DustToTreasury;
impl frame_support::traits::OnUnbalanced<pallet_balances::CreditOf<Runtime, ()>>
	for DustToTreasury
{
	fn on_nonzero_unbalanced(amount: pallet_balances::CreditOf<Runtime, ()>) {
		use frame_support::traits::fungible::Balanced;
		use sp_runtime::traits::AccountIdConversion;

		let treasury_account = TreasuryPalletId::get().into_account_truncating();

		let _ = Balances::resolve(&treasury_account, amount);
	}
}

// --- Balances ---

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type Balance = Balance;
	type RuntimeEvent = RuntimeEvent;
	/// Dust below existential deposit is swept into the on-chain treasury
	/// instead of being burned, preserving total supply accounting.
	type DustRemoval = DustToTreasury;
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeFreezeReason = ();
	type RuntimeHoldReason = RuntimeHoldReason;
}

// --- Utility (batch calls, etc.) ---

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

// --- Vesting ---

parameter_types! {
	pub const MinVestedTransfer: Balance = 100 * UNIT;
	pub UnvestedFundsAllowedWithdrawReasons: frame_support::traits::WithdrawReasons =
		frame_support::traits::WithdrawReasons::except(
			frame_support::traits::WithdrawReasons::TRANSFER | frame_support::traits::WithdrawReasons::RESERVE
		);
}

impl pallet_vesting::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type BlockNumberToBalance = sp_runtime::traits::ConvertInto;
	type MinVestedTransfer = MinVestedTransfer;
	type WeightInfo = pallet_vesting::weights::SubstrateWeight<Runtime>;
	type UnvestedFundsAllowedWithdrawReasons = UnvestedFundsAllowedWithdrawReasons;
	type BlockNumberProvider = System;
	const MAX_VESTING_SCHEDULES: u32 = 28;
}

// --- ink! / WASM Smart Contracts (pallet_contracts) ---
//
// Developers deploy ink! contracts to this runtime via `cargo contract` or
// Contracts UI. The fee model charges per storage item and per byte stored,
// encouraging contracts to clean up unused state.

/// Storage deposit helper — charges 1 XNET per item and 0.01 XNET per byte.
/// Encourages contracts to free storage they no longer need.
pub const fn deposit(items: u32, bytes: u32) -> Balance {
	(items as Balance * 1_000_000_000_000_000_000)
		.saturating_add((bytes as Balance) * 10_000_000_000_000_000)
}

parameter_types! {
	pub const DepositPerItem: Balance = deposit(1, 0);
	pub const DepositPerByte: Balance = deposit(0, 1);
	pub const DefaultDepositLimit: Balance = deposit(1024, 1024 * 1024);
	pub Schedule: pallet_contracts::Schedule<Runtime> = Default::default();
	pub const CodeHashLockupDepositPercent: sp_runtime::Perbill = sp_runtime::Perbill::from_percent(30);
	pub const MaxTransientStorageSize: u32 = 1024 * 1024;
	pub const MaxDelegateDependencies: u32 = 32;
}

impl pallet_contracts::Config for Runtime {
	type Time = Timestamp;
	type Randomness = pallet_babe::RandomnessFromOneEpochAgo<Runtime>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	/// Allow contracts to dispatch any call that a regular signed account can make.
	/// Using `Nothing` here would silently block all cross-contract and contract→pallet
	/// calls, causing confusing errors for ink! developers.
	/// Contracts cannot dispatch arbitrary runtime calls. This prevents
	/// a malicious contract from calling Sudo, Staking, or other privileged
	/// pallets. Contracts interact with native tokens via regular transfers.
	type CallFilter = frame_support::traits::Nothing;
	type DepositPerItem = DepositPerItem;
	type DepositPerByte = DepositPerByte;
	type DefaultDepositLimit = DefaultDepositLimit;
	type WeightPrice = pallet_transaction_payment::Pallet<Self>;
	type WeightInfo = pallet_contracts::weights::SubstrateWeight<Self>;
	type ChainExtension = ();
	type Schedule = Schedule;
	type CallStack = [pallet_contracts::Frame<Self>; 5];
	type AddressGenerator = pallet_contracts::DefaultAddressGenerator;
	type MaxCodeLen = frame_support::traits::ConstU32<125_000>;
	type MaxStorageKeyLen = frame_support::traits::ConstU32<128>;
	type UnsafeUnstableInterface = frame_support::traits::ConstBool<false>;
	type MaxDebugBufferLen = frame_support::traits::ConstU32<{ 2 * 1024 * 1024 }>;
	type Environment = ();
	type RuntimeHoldReason = RuntimeHoldReason;
	type CodeHashLockupDepositPercent = CodeHashLockupDepositPercent;
	type MaxTransientStorageSize = MaxTransientStorageSize;
	type MaxDelegateDependencies = MaxDelegateDependencies;
	type UploadOrigin = frame_system::EnsureSigned<Self::AccountId>;
	type InstantiateOrigin = frame_system::EnsureSigned<Self::AccountId>;
	type ApiVersion = ();
	type Migrations = ();
	type Debug = ();
	type Xcm = ();
}

// --- Transaction Payment ---

// Fee handler: splits each transaction fee 75/25 between the on-chain treasury
// and the block author. The treasury portion funds future development; the
// author portion incentivises validators to include transactions quickly.

parameter_types! {
	/// Ekotizimni rivojlantirish (Bounty va Dasturchilar) uchun Grant Hovuzi ID'si
	pub const GrantPalletId: PalletId = PalletId(*b"py/grant");
}

/// Fee handler: Komissiyalarni 3 ga bo'ladi — 60% G'aznaga, 15% Grant Hovuziga, 25% Validatorga
pub struct DealWithFees;
impl frame_support::traits::OnUnbalanced<Credit<AccountId, pallet_balances::Pallet<Runtime>>>
	for DealWithFees
{
	fn on_unbalanced(fees: Credit<AccountId, pallet_balances::Pallet<Runtime>>) {
		use frame_support::traits::fungible::Balanced;
		use frame_support::traits::Imbalance;
		use sp_runtime::traits::AccountIdConversion; // Hamyon manziliga aylantirish uchun

		// 1. Avval pulni 2 ga bo'lamiz: 75% G'azna+Grant uchun, 25% Validator uchun
		let (to_treasury_and_grant, to_author) = fees.ration(75, 25);

		// 2. Endi o'sha 75% ning ichidan: 80% (umumiy 60%) G'aznaga, 20% (umumiy 15%) Grantga bo'lamiz
		let (to_treasury, to_grant) = to_treasury_and_grant.ration(80, 20);

		// --- PULLARNI HAMYONLARGA TUSHIRAMIZ ---

		// 60% Asosiy G'aznaga (Treasury) tushadi
		let treasury_account = TreasuryPalletId::get().into_account_truncating();
		let _ = <pallet_balances::Pallet<Runtime>>::resolve(&treasury_account, to_treasury);

		// 15% Grant Hovuziga (Grant Pool) tushadi
		let grant_account = GrantPalletId::get().into_account_truncating();
		let _ = <pallet_balances::Pallet<Runtime>>::resolve(&grant_account, to_grant);

		// 25% Blokni yasagan Validatorga (Minerga) tushadi
		if let Some(author) = Authorship::author() {
			let _ = <pallet_balances::Pallet<Runtime>>::resolve(&author, to_author);
		}
	}
}

parameter_types! {
	pub const TransactionByteFee: Balance = 10_000_000_000_000;
	pub const TargetBlockFullness: sp_runtime::Perquintill = sp_runtime::Perquintill::from_percent(25);
	pub AdjustmentVariable: pallet_transaction_payment::Multiplier =
		pallet_transaction_payment::Multiplier::from_rational(1, 100_000);
	pub MinimumMultiplier: pallet_transaction_payment::Multiplier =
		pallet_transaction_payment::Multiplier::from_u32(1);
	pub MaximumMultiplier: pallet_transaction_payment::Multiplier =
		sp_runtime::traits::Bounded::max_value();
}

/// Converts execution weight to a fee denominated in XNET tokens.
/// Uses a linear multiplier so that the base extrinsic (~110_000 ref_time)
/// costs approximately 0.0001 XNET (= 100_000_000_000_000 planck).
pub struct WeightToFeeStruct;
impl frame_support::weights::WeightToFee for WeightToFeeStruct {
	type Balance = Balance;
	fn weight_to_fee(weight: &Weight) -> Self::Balance {
		// 1 ref_time unit ≈ 1_000_000_000 planck (1 Gwei equivalent)
		// This yields ~0.0001 XNET per basic extrinsic, discouraging spam
		// while keeping fees affordable for regular users.
		(weight.ref_time() as Balance).saturating_mul(1_000_000_000)
	}
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = pallet_transaction_payment::FungibleAdapter<Balances, DealWithFees>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = WeightToFeeStruct;
	type LengthToFee = frame_support::weights::ConstantMultiplier<Balance, TransactionByteFee>;
	type FeeMultiplierUpdate = pallet_transaction_payment::TargetedFeeAdjustment<
		Self,
		TargetBlockFullness,
		AdjustmentVariable,
		MinimumMultiplier,
		MaximumMultiplier,
	>;
}

// --- Authorship / ImOnline / Offences ---

impl pallet_authorship::Config for Runtime {
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type EventHandler = (Staking, ImOnline);
}

impl pallet_im_online::Config for Runtime {
	type AuthorityId = ImOnlineId;
	type RuntimeEvent = RuntimeEvent;
	type NextSessionRotation = Babe;
	type ValidatorSet = Historical;
	type ReportUnresponsiveness = Offences;
	type UnsignedPriority = ConstU64<{ u64::MAX }>;
	type WeightInfo = pallet_im_online::weights::SubstrateWeight<Runtime>;
	type MaxKeys = ConstU32<10_000>;
	type MaxPeerInHeartbeats = ConstU32<10_000>;
}

impl pallet_offences::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type IdentificationTuple = pallet_session::historical::IdentificationTuple<Self>;
	type OnOffenceHandler = Staking;
}

// --- NPoS Staking ---

parameter_types! {
	pub const SessionsPerEra: sp_staking::SessionIndex = 24;
	pub const BondingDuration: sp_staking::EraIndex = 28;
	pub const SlashDeferDuration: sp_staking::EraIndex = 7;
	pub const MaxNominatorRewardedPerValidator: u32 = 64;
	pub const OffendingValidatorsThreshold: Perbill = Perbill::from_percent(17);
	pub const HistoryDepth: u32 = 84;
}

/// Benchmarking parameters for staking — keep conservative to avoid benchmark timeouts.
pub struct StakingBenchmarkingConfig;
impl pallet_staking::BenchmarkingConfig for StakingBenchmarkingConfig {
	type MaxNominators = ConstU32<1000>;
	type MaxValidators = ConstU32<1000>;
}

// Inflation curve: minimum 2.5% annual, maximum 10% annual.
// Inflation peaks when 50% of supply is staked and tapers off toward the minimum
// as the staked ratio moves away from the ideal in either direction.
pallet_staking_reward_curve::build! {
	const REWARD_CURVE: sp_runtime::curve::PiecewiseLinear<'static> = curve!(
		min_inflation: 0_025_000,   // 2.5% minimum annual inflation
		max_inflation: 0_100_000,   // 10%  maximum annual inflation
		ideal_stake:   0_500_000,   // 50%  of supply staked is the target
		falloff:       0_050_000,
		max_piece_count: 40,
		test_precision:  0_005_000,
	);
}

parameter_types! {
	pub const RewardCurve: &'static sp_runtime::curve::PiecewiseLinear<'static> = &REWARD_CURVE;
}

/// Sequential Phragmén election solver used to select the active validator set
/// on-chain. It favours proportional representation over raw stake weight.
pub struct OnChainSeqPhragmen;
impl onchain::Config for OnChainSeqPhragmen {
	type System = Runtime;
	type Solver = SequentialPhragmen<AccountId, sp_runtime::Perbill>;
	type DataProvider = Staking;
	type WeightInfo = frame_election_provider_support::weights::SubstrateWeight<Runtime>;
	type MaxWinners = ConstU32<100>;
	type Bounds = DefaultElectionBounds;
}

impl pallet_staking::Config for Runtime {
	type Currency = Balances;
	type CurrencyBalance = Balance;
	type UnixTime = Timestamp;
	type RuntimeEvent = RuntimeEvent;
	type Slash = Treasury;
	type Reward = ();
	type SessionsPerEra = SessionsPerEra;
	type BondingDuration = BondingDuration;
	type SlashDeferDuration = SlashDeferDuration;
	type AdminOrigin = frame_system::EnsureRoot<AccountId>;
	type SessionInterface = Self;
	/// Staking inflation is disabled — XNET uses a single minting source:
	/// the block-reward pallet with Bitcoin-style halving. Setting EraPayout
	/// to () ensures the staking pallet does not mint additional tokens,
	/// so total supply stays within the 53M hard cap.
	type EraPayout = ();
	type NextNewSession = Session;
	type MaxExposurePageSize = ConstU32<256>;
	type ElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GenesisElectionProvider = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type VoterList = pallet_staking::UseNominatorsAndValidatorsMap<Runtime>;
	type TargetList = pallet_staking::UseValidatorsMap<Runtime>;
	type NominationsQuota = pallet_staking::FixedNominationsQuota<16>;
	type MaxUnlockingChunks = ConstU32<32>;
	type HistoryDepth = HistoryDepth;
	type EventListeners = ();
	type BenchmarkingConfig = StakingBenchmarkingConfig;
	type WeightInfo = pallet_staking::weights::SubstrateWeight<Runtime>;
	type CurrencyToVote = U128CurrencyToVote;
	type RewardRemainder = Treasury;
	type MaxControllersInDeprecationBatch = ConstU32<100>;
	type DisablingStrategy = pallet_staking::UpToLimitDisablingStrategy<17>;
}

impl pallet_session::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ValidatorId = <Self as frame_system::Config>::AccountId;
	type ValidatorIdOf = pallet_staking::StashOf<Self>;
	type ShouldEndSession = Babe;
	type NextSessionRotation = Babe;
	type SessionManager = pallet_session::historical::NoteHistoricalRoot<Self, Staking>;
	type SessionHandler = (Babe, Grandpa, ImOnline);
	type Keys = opaque::SessionKeys;
	type WeightInfo = pallet_session::weights::SubstrateWeight<Runtime>;
}

impl pallet_session::historical::Config for Runtime {
	type FullIdentification = pallet_staking::Exposure<AccountId, Balance>;
	type FullIdentificationOf = pallet_staking::ExposureOf<Runtime>;
}

// --- Treasury ---

parameter_types! {
	pub const ProposalBond: Permill = Permill::from_percent(5);
	pub const ProposalBondMinimum: Balance = 100 * UNIT;
	pub const SpendPeriod: BlockNumber = DAYS;
	pub const Burn: Permill = Permill::from_percent(0);
	pub const MaxApprovals: u32 = 100;
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
}

parameter_types! {
	pub TreasuryAccount: AccountId = TreasuryPalletId::get().into_account_truncating();
}

use sp_runtime::traits::AccountIdConversion;

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_treasury::weights::SubstrateWeight<Runtime>;
	type MaxApprovals = MaxApprovals;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = AccountIdLookup<AccountId, ()>;
	type Paymaster = frame_support::traits::tokens::pay::PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = frame_support::traits::tokens::UnityAssetBalanceConversion;
	type PayoutPeriod = ConstU32<60>;
	type SpendPeriod = SpendPeriod;
	type Burn = Burn;
	type BurnDestination = ();
	type SpendFunds = ();
	type SpendOrigin = frame_support::traits::NeverEnsureOrigin<u128>;
	type RejectOrigin = frame_system::EnsureRoot<AccountId>;
	type BenchmarkHelper = ();
}

// --- Sudo (bootstrap governance — replace with on-chain governance before mainnet) ---

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

// --- ZK Verifier (Groth16 on-chain proof verification) ---

parameter_types! {
    /// Maximum number of ZK proofs that can be verified in a single block.
    /// Prevents DoS attacks via proof spam.
    pub const MaxProofsPerBlock: u32 = 20;

    /// Maximum number of public inputs per proof.
    /// Standard Groth16 circuits rarely need more than 16.
    pub const MaxPublicInputs: u32 = 16;
}

impl pallet_zk_verifier::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type WeightInfo = pallet_zk_verifier::weights::SubstrateWeight<Runtime>;
    type MaxProofsPerBlock = MaxProofsPerBlock;
    type MaxPublicInputs = MaxPublicInputs;
}

// --- Election Provider Multi Phase (off-chain validator election) ---

frame_election_provider_support::generate_solution_type!(
	#[compact]
	pub struct NposSolution16::<
		VoterIndex = u32,
		TargetIndex = u16,
		Accuracy = sp_runtime::PerU16,
		MaxVoters = ConstU32::<2400>,
	>(16)
);

parameter_types! {
	pub MaxElectionWeight: Weight = BlockWeights::get().max_block;
}

/// Off-chain miner configuration. Limits solution weight and voter count so
/// that unsigned election submissions stay within block limits.
pub struct CustomMinerConfig;
impl pallet_election_provider_multi_phase::MinerConfig for CustomMinerConfig {
	type AccountId = AccountId;
	type MaxLength = ConstU32<256>;
	type MaxWeight = MaxElectionWeight;
	type MaxVotesPerVoter = ConstU32<16>;
	type MaxWinners = ConstU32<100>;
	type Solution = NposSolution16;
	fn solution_weight(v: u32, t: u32, a: u32, d: u32) -> Weight {
		// Rough approximation: each voter × target pair plus assignments
		// and desired winners contribute to execution cost.
		Weight::from_parts(
			(v.saturating_mul(t).saturating_add(a).saturating_add(d)) as u64 * 1_000,
			0,
		)
	}
}

/// Converts the number of election data items into a XNET deposit amount.
/// Larger snapshots cost more to submit, deterring spam.
pub struct DepositBase;
impl Convert<usize, Balance> for DepositBase {
	fn convert(x: usize) -> Balance {
		(x as u128) * UNIT
	}
}

//  ---NEW---
pub struct ElectionMultiPhaseBenchmarkConfig;

impl pallet_election_provider_multi_phase::BenchmarkingConfig
	for ElectionMultiPhaseBenchmarkConfig
{
	const VOTERS: [u32; 2] = [1000, 2000];
	const TARGETS: [u32; 2] = [100, 200];
	const ACTIVE_VOTERS: [u32; 2] = [500, 1000];
	const DESIRED_TARGETS: [u32; 2] = [50, 100];

	// MISSING ITEMS:
	const SNAPSHOT_MAXIMUM_VOTERS: u32 = 1000;
	const MINER_MAXIMUM_VOTERS: u32 = 1000;
	const MAXIMUM_TARGETS: u32 = 300;
}

impl pallet_election_provider_multi_phase::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type EstimateCallFee = TransactionPayment;
	type SignedPhase = ();
	type UnsignedPhase = ();
	type SignedMaxSubmissions = ConstU32<10>;
	type SignedRewardBase = ConstU128<UNIT>;
	type SignedDepositBase = DepositBase;
	type SignedDepositByte = ConstU128<MICROUNIT>;
	type SignedDepositWeight = ();
	type SignedMaxWeight = ();
	type SlashHandler = ();
	type RewardHandler = ();
	type BetterSignedThreshold = ();
	type OffchainRepeat = ();
	type MinerTxPriority = ConstU64<{ u64::MAX }>;
	type MinerConfig = CustomMinerConfig;
	type Solver = SequentialPhragmen<
		AccountId,
		pallet_election_provider_multi_phase::SolutionAccuracyOf<Runtime>,
	>;
	type WeightInfo = pallet_election_provider_multi_phase::weights::SubstrateWeight<Runtime>;
	type MaxWinners = ConstU32<100>;
	type ElectionBounds = DefaultElectionBounds;
	type SignedMaxRefunds = ConstU32<0>;
	type DataProvider = Staking;
	type Fallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type GovernanceFallback = onchain::OnChainExecution<OnChainSeqPhragmen>;
	type ForceOrigin = EnsureRoot<AccountId>;
	type BenchmarkingConfig = ElectionMultiPhaseBenchmarkConfig;
}

// --- Block Reward (custom pallet with Bitcoin-style halving) ---

parameter_types! {
	/// Starting block reward before any halving occurs.
	pub const InitialBlockReward: Balance = BLOCK_REWARD;
	/// Number of blocks between halvings (~4 years at 6-second blocks).
	pub const HalvingInterval: u32 = 21_038_400;
	/// Absolute token supply ceiling — 53,000,000 XNET.
	/// Minting stops permanently once total issuance reaches this value.
	pub const MaxSupply: Balance = MAX_SUPPLY;
}

impl pallet_block_reward::Config for Runtime {
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type FindAuthor = pallet_session::FindAccountFromAuthorIndex<Self, Babe>;
	type InitialBlockReward = InitialBlockReward;
	type HalvingInterval = HalvingInterval;
	/// Enforce the 53,000,000 XNET hard cap at every minting step.
	type MaxSupply = MaxSupply;
}

// --- Election Bounds (caps snapshot size to protect block execution time) ---

parameter_types! {
	pub const DefaultElectionBounds: frame_election_provider_support::bounds::ElectionBounds =
		frame_election_provider_support::bounds::ElectionBounds {
			voters: frame_election_provider_support::DataProviderBounds {
				count: Some(frame_election_provider_support::bounds::CountBound(2500)),
				size: Some(frame_election_provider_support::bounds::SizeBound(u32::MAX))
			},
			targets: frame_election_provider_support::DataProviderBounds {
				count: Some(frame_election_provider_support::bounds::CountBound(100)),
				size: Some(frame_election_provider_support::bounds::SizeBound(u32::MAX))
			},
		};
}

// =============================================================================
// EVM Stack (Frontier — Ethereum-compatible execution layer)
// =============================================================================

/// Maps the BABE authority index to an Ethereum-style H160 author address.
/// Takes bytes [4..24] of the raw Sr25519 public key — the same convention
/// used by Moonbeam and other Frontier-based chains.
pub struct FindAuthorTruncated<F>(sp_std::marker::PhantomData<F>);
impl<F: FindAuthor<u32>> FindAuthor<H160> for FindAuthorTruncated<F> {
	fn find_author<'a, I>(digests: I) -> Option<H160>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		if let Some(author_index) = F::find_author(digests) {
			let authority_id = Babe::authorities()[author_index as usize].clone();
			return Some(H160::from_slice(&sp_core::ByteArray::to_raw_vec(&authority_id.0)[4..24]));
		}
		None
	}
}

/// Maximum proof-size (storage growth) allowed per block, in bytes.
/// This caps how much state an EVM block can write, guarding against
/// unbounded state growth that would bloat all full nodes.
const MAX_STORAGE_GROWTH: u64 = 400 * 1024;

parameter_types! {
	/// Base fee starts at 1 Gwei — consistent with the EIP-1559 minimum.
	pub const DefaultBaseFeePerGas: U256 = U256([1_000_000_000u64, 0, 0, 0]);
	/// EIP-1559 elasticity: base fee can grow/shrink by 12.5% per block.
	pub DefaultElasticity: sp_runtime::Permill = sp_runtime::Permill::from_parts(125_000);
	/// Maximum gas allowed per block (75M) — roughly equivalent to Ethereum mainnet.
	pub BlockGasLimit: U256 = U256::from(75_000_000u64);
	/// Substrate weight units consumed per EVM gas unit.
	pub WeightPerGas: Weight = Weight::from_parts(20_000, 0);
	/// Ratio of block gas limit to maximum storage growth per block.
	/// Controls how aggressively the EVM runner charges for storage writes.
	pub const GasLimitStorageGrowthRatio: u64 = 75_000_000u64.saturating_div(MAX_STORAGE_GROWTH);
}

parameter_types! {
	/// The XnetPrecompiles instance is a zero-sized type — instantiation is free.
	pub PrecompilesValue: XnetPrecompiles<Runtime> = XnetPrecompiles::new();
}

impl pallet_evm::Config for Runtime {
	type FeeCalculator = BaseFee;
	type GasWeightMapping = pallet_evm::FixedGasWeightMapping<Self>;
	type WeightPerGas = WeightPerGas;
	type BlockHashMapping = pallet_ethereum::EthereumBlockHashMapping<Self>;
	type CallOrigin = EnsureAddressTruncated;
	type WithdrawOrigin = EnsureAddressTruncated;
	type AddressMapping = HashedAddressMapping<BlakeTwo256>;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	/// Full Ethereum precompile set: ECRecover, SHA-256, RIPEMD-160, Identity,
	/// ModExp, BN128Add/Mul/Pairing (ZK-SNARKs), and BLAKE2F.
	type PrecompilesType = XnetPrecompiles<Self>;
	type PrecompilesValue = PrecompilesValue;
	type ChainId = ConstU64<2009>;
	type BlockGasLimit = BlockGasLimit;
	type Runner = pallet_evm::runner::stack::Runner<Self>;
	/// Route EVM gas fees through `DealWithFees` — 75% treasury, 25% block author.
	/// This ensures EVM transactions contribute to the same fee economy as
	/// Substrate-native extrinsics.
	type OnChargeTransaction = pallet_evm::EVMFungibleAdapter<Balances, DealWithFees>;
	type OnCreate = ();
	type FindAuthor = FindAuthorTruncated<Babe>;
	type GasLimitPovSizeRatio = ConstU64<4>;
	/// Limits storage writes per block relative to the block gas limit.
	/// Prevents storage-heavy EVM transactions from causing unbounded node growth.
	type GasLimitStorageGrowthRatio = GasLimitStorageGrowthRatio;
	/// Delegates account nonce management to `frame_system`.
	/// This ties EVM account nonces to the standard Substrate account system.
	type AccountProvider = pallet_evm::FrameSystemAccountProvider<Self>;
	type Timestamp = Timestamp;
	type WeightInfo = pallet_evm::weights::SubstrateWeight<Self>;
}

impl pallet_ethereum::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	/// Computes the Ethereum state root from the runtime version digest.
	/// `Self::Version` implements `Get<RuntimeVersion>` via `frame_system::Config`,
	/// satisfying the bound required by `IntermediateStateRoot`.
	type StateRoot = pallet_ethereum::IntermediateStateRoot<<Self as frame_system::Config>::Version>;
	type PostLogContent = ();
	type ExtraDataLength = ConstU32<30>;
}

/// EIP-1559 base fee adjustment thresholds. The fee rises when blocks are above
/// 50% full and falls when they are below, targeting stable block utilisation.
pub struct CustomBaseFeeThreshold;
impl pallet_base_fee::BaseFeeThreshold for CustomBaseFeeThreshold {
	fn lower() -> sp_runtime::Permill {
		sp_runtime::Permill::zero()
	}
	fn ideal() -> sp_runtime::Permill {
		sp_runtime::Permill::from_parts(500_000)
	}
	fn upper() -> sp_runtime::Permill {
		sp_runtime::Permill::from_parts(1_000_000)
	}
}

impl pallet_base_fee::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Threshold = CustomBaseFeeThreshold;
	type DefaultBaseFeePerGas = DefaultBaseFeePerGas;
	type DefaultElasticity = DefaultElasticity;
}

// --- Off-chain Worker / Signed Transaction Helpers ---

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = RuntimeCall;
}

impl frame_system::offchain::SigningTypes for Runtime {
	type Public = <Signature as Verify>::Signer;
	type Signature = Signature;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		public: <Signature as Verify>::Signer,
		account: AccountId,
		nonce: Nonce,
	) -> Option<(
		RuntimeCall,
		<UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload,
	)> {
		let tip = 0;
		let period =
			BlockHashCount::get().checked_next_power_of_two().map(|c| c / 2).unwrap_or(2) as u64;
		let current_block = System::block_number().saturating_sub(1);
		let era = generic::Era::mortal(period, current_block.into());
		let extra = (
			frame_system::CheckNonZeroSender::<Runtime>::new(),
			frame_system::CheckSpecVersion::<Runtime>::new(),
			frame_system::CheckTxVersion::<Runtime>::new(),
			frame_system::CheckGenesis::<Runtime>::new(),
			frame_system::CheckEra::<Runtime>::from(era),
			frame_system::CheckNonce::<Runtime>::from(nonce),
			frame_system::CheckWeight::<Runtime>::new(),
			pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
		);
		let raw_payload = SignedPayload::new(call, extra).map_err(|_| ()).ok()?;
		let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
		let (call, extra, _) = raw_payload.deconstruct();
		Some((call, (sp_runtime::MultiAddress::Id(account), signature, extra)))
	}
}

// =============================================================================
// Runtime Construction
// =============================================================================

construct_runtime!(
	pub struct Runtime {
		// --- Core ---
		System: frame_system,
		Timestamp: pallet_timestamp,

		// --- Consensus ---
		Babe: pallet_babe,
		Grandpa: pallet_grandpa,
		Authorship: pallet_authorship,
		ImOnline: pallet_im_online,
		Offences: pallet_offences,

		// --- Balances & Payment ---
		Balances: pallet_balances,
		TransactionPayment: pallet_transaction_payment,

		// --- Utility ---
		Utility: pallet_utility,
		Vesting: pallet_vesting,

		// --- Staking & Session ---
		Staking: pallet_staking,
		Session: pallet_session,
		Historical: pallet_session::historical::{Pallet},
		ElectionProviderMultiPhase: pallet_election_provider_multi_phase,

		// --- Governance & Treasury ---
		Treasury: pallet_treasury,
		Sudo: pallet_sudo,

		// --- Custom ---
		BlockReward: pallet_block_reward,

		// --- Privacy ---
		ZkVerifier: pallet_zk_verifier,

		// --- Wasm Smart Contracts ---
		Contracts: pallet_contracts,

		// --EVM Stack ---
		Ethereum: pallet_ethereum,
		EVM: pallet_evm,
		BaseFee: pallet_base_fee,
	}
);

// =============================================================================
// Type Aliases & Executive
// =============================================================================

pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

pub type UncheckedExtrinsic =
	fp_self_contained::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
>;

pub use pallet_staking::StakerStatus;

pub const BABE_GENESIS_EPOCH_CONFIG: sp_consensus_babe::BabeEpochConfiguration =
	sp_consensus_babe::BabeEpochConfiguration {
		c: (1, 4),
		// VRF-based secondary slots prevent slot-number prediction attacks
		// that are possible with plain secondary slots.
		allowed_slots: sp_consensus_babe::AllowedSlots::PrimaryAndSecondaryVRFSlots,
	};

// --- Ethereum Transaction Converter (bridges Frontier RPC to Substrate extrinsics) ---

/// Converts a raw Ethereum transaction into a Substrate `UncheckedExtrinsic`
/// so the transaction pool and block builder can handle it uniformly.
pub struct TransactionConverter;

impl fp_rpc::ConvertTransaction<UncheckedExtrinsic> for TransactionConverter {
	fn convert_transaction(&self, transaction: pallet_ethereum::Transaction) -> UncheckedExtrinsic {
		UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		)
	}
}

impl fp_rpc::ConvertTransaction<sp_runtime::OpaqueExtrinsic> for TransactionConverter {
	fn convert_transaction(
		&self,
		transaction: pallet_ethereum::Transaction,
	) -> sp_runtime::OpaqueExtrinsic {
		let extrinsic = UncheckedExtrinsic::new_unsigned(
			pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
		);
		let encoded = extrinsic.encode();
		sp_runtime::OpaqueExtrinsic::decode(&mut &encoded[..])
			.expect("Encoded extrinsic is always valid")
	}
}

// --- Self-Contained Transactions (Ethereum txs carry their own signature/auth) ---

impl fp_self_contained::SelfContainedCall for RuntimeCall {
	type SignedInfo = H160;

	fn is_self_contained(&self) -> bool {
		match self {
			RuntimeCall::Ethereum(call) => call.is_self_contained(),
			_ => false,
		}
	}

	fn check_self_contained(
		&self,
	) -> Option<Result<Self::SignedInfo, sp_runtime::transaction_validity::TransactionValidityError>>
	{
		match self {
			RuntimeCall::Ethereum(call) => call.check_self_contained(),
			_ => None,
		}
	}

	fn validate_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<sp_runtime::transaction_validity::TransactionValidity> {
		match self {
			RuntimeCall::Ethereum(call) => call.validate_self_contained(info, dispatch_info, len),
			_ => None,
		}
	}

	fn pre_dispatch_self_contained(
		&self,
		info: &Self::SignedInfo,
		dispatch_info: &sp_runtime::traits::DispatchInfoOf<RuntimeCall>,
		len: usize,
	) -> Option<Result<(), sp_runtime::transaction_validity::TransactionValidityError>> {
		match self {
			RuntimeCall::Ethereum(call) => {
				call.pre_dispatch_self_contained(info, dispatch_info, len)
			},
			_ => None,
		}
	}

	fn apply_self_contained(
		self,
		info: Self::SignedInfo,
	) -> Option<sp_runtime::DispatchResultWithInfo<sp_runtime::traits::PostDispatchInfoOf<Self>>> {
		match self {
			call @ RuntimeCall::Ethereum(pallet_ethereum::Call::transact { .. }) => {
				Some(call.dispatch(RuntimeOrigin::from(
					pallet_ethereum::RawOrigin::EthereumTransaction(info),
				)))
			},
			_ => None,
		}
	}
}

// =============================================================================
// Runtime API Implementations
// =============================================================================

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion { VERSION }
		fn execute_block(block: Block) { Executive::execute_block(block); }
		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> sp_std::vec::Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}
		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}
		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}
		fn check_inherents(block: Block, data: sp_inherents::InherentData) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_babe::BabeApi<Block> for Runtime {
		fn configuration() -> sp_consensus_babe::BabeConfiguration {
			let epoch_config = Babe::epoch_config().unwrap_or(BABE_GENESIS_EPOCH_CONFIG);
			sp_consensus_babe::BabeConfiguration {
				slot_duration: Babe::slot_duration(),
				epoch_length: EpochDuration::get(),
				c: epoch_config.c,
				authorities: Babe::authorities().to_vec(),
				randomness: Babe::randomness(),
				allowed_slots: epoch_config.allowed_slots,
			}
		}
		fn current_epoch_start() -> sp_consensus_babe::Slot { Babe::current_epoch_start() }
		fn current_epoch() -> sp_consensus_babe::Epoch { Babe::current_epoch() }
		fn next_epoch() -> sp_consensus_babe::Epoch { Babe::next_epoch() }
		fn generate_key_ownership_proof(
			_slot: sp_consensus_babe::Slot,
			authority_id: sp_consensus_babe::AuthorityId,
		) -> Option<sp_consensus_babe::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_babe::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_babe::OpaqueKeyOwnershipProof::new)
		}
		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_babe::EquivocationProof<<Block as BlockT>::Header>,
			key_owner_proof: sp_consensus_babe::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;
			Babe::submit_unsigned_equivocation_report(equivocation_proof, key_owner_proof)
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}
		fn decode_session_keys(encoded: Vec<u8>) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}
		fn current_set_id() -> sp_consensus_grandpa::SetId { Grandpa::current_set_id() }
		fn submit_report_equivocation_unsigned_extrinsic(
			equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			let key_owner_proof = key_owner_proof.decode()?;
			Grandpa::submit_unsigned_equivocation_report(equivocation_proof, key_owner_proof)
		}
		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			authority_id: sp_consensus_grandpa::AuthorityId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			Historical::prove((sp_consensus_grandpa::KEY_TYPE, authority_id))
				.map(|p| p.encode())
				.map(sp_consensus_grandpa::OpaqueKeyOwnershipProof::new)
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance { TransactionPayment::weight_to_fee(weight) }
		fn query_length_to_fee(length: u32) -> Balance { TransactionPayment::length_to_fee(length) }
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			frame_support::genesis_builder_helper::build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			frame_support::genesis_builder_helper::get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}

	impl pallet_contracts::ContractsApi<Block, AccountId, Balance, BlockNumber, Hash, frame_system::EventRecord<RuntimeEvent, Hash>> for Runtime {
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			input_data: Vec<u8>,
		) -> pallet_contracts::ContractExecResult<Balance, frame_system::EventRecord<RuntimeEvent, Hash>> {
			let limit = gas_limit.unwrap_or(Weight::MAX);
			Contracts::bare_call(
				origin,
				dest,
				value,
				limit,
				storage_deposit_limit,
				input_data,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
				pallet_contracts::Determinism::Enforced,
			)
		}

		fn instantiate(
			origin: AccountId,
			value: Balance,
			gas_limit: Option<Weight>,
			storage_deposit_limit: Option<Balance>,
			code: pallet_contracts::Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> pallet_contracts::ContractInstantiateResult<AccountId, Balance, frame_system::EventRecord<RuntimeEvent, Hash>> {
			let limit = gas_limit.unwrap_or(Weight::MAX);
			Contracts::bare_instantiate(
				origin,
				value,
				limit,
				storage_deposit_limit,
				code,
				data,
				salt,
				pallet_contracts::DebugInfo::UnsafeDebug,
				pallet_contracts::CollectEvents::UnsafeCollect,
			)
		}

		fn upload_code(
			origin: AccountId,
			code: Vec<u8>,
			storage_deposit_limit: Option<Balance>,
			determinism: pallet_contracts::Determinism,
		) -> pallet_contracts::CodeUploadResult<Hash, Balance> {
			Contracts::bare_upload_code(origin, code, storage_deposit_limit, determinism)
		}

		fn get_storage(
			address: AccountId,
			key: Vec<u8>,
		) -> pallet_contracts::GetStorageResult {
			Contracts::get_storage(address, key)
		}
	}
	impl fp_rpc::EthereumRuntimeRPCApi<Block> for Runtime {
		fn chain_id() -> u64 {
			<Runtime as pallet_evm::Config>::ChainId::get()
		}

		fn account_basic(address: H160) -> EVMAccount {
			let (account, _) = EVM::account_basic(&address);
			account
		}

		fn gas_price() -> U256 {
			let (gas_price, _) = <Runtime as pallet_evm::Config>::FeeCalculator::min_gas_price();
			gas_price
		}

		fn account_code_at(address: H160) -> Vec<u8> {
			pallet_evm::AccountCodes::<Runtime>::get(address)
		}

		fn author() -> H160 {
			<pallet_evm::Pallet<Runtime>>::find_author()
		}

		fn storage_at(address: H160, index: U256) -> H256 {
			let mut tmp = [0u8; 32];
			index.to_big_endian(&mut tmp);
			pallet_evm::AccountStorages::<Runtime>::get(address, H256::from_slice(&tmp[..]))
		}

		fn call(
			from: H160,
			to: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			_estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CallInfo, sp_runtime::DispatchError> {
			let config = <Runtime as pallet_evm::Config>::config();
			<Runtime as pallet_evm::Config>::Runner::call(
				from,
				to,
				data,
				value,
				gas_limit.low_u64(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				None,
				None,
				config,
			).map_err(|err| err.error.into())
		}

		fn create(
			from: H160,
			data: Vec<u8>,
			value: U256,
			gas_limit: U256,
			max_fee_per_gas: Option<U256>,
			max_priority_fee_per_gas: Option<U256>,
			nonce: Option<U256>,
			_estimate: bool,
			access_list: Option<Vec<(H160, Vec<H256>)>>,
		) -> Result<pallet_evm::CreateInfo, sp_runtime::DispatchError> {
			let config = <Runtime as pallet_evm::Config>::config();
			<Runtime as pallet_evm::Config>::Runner::create(
				from,
				data,
				value,
				gas_limit.low_u64(),
				max_fee_per_gas,
				max_priority_fee_per_gas,
				nonce,
				access_list.unwrap_or_default(),
				false,
				true,
				None,
				None,
				config,
			).map_err(|err| err.error.into())
		}

		fn current_transaction_statuses() -> Option<Vec<TransactionStatus>> {
			pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get()
		}

		fn current_block() -> Option<pallet_ethereum::Block> {
			pallet_ethereum::CurrentBlock::<Runtime>::get()
		}

		fn current_receipts() -> Option<Vec<pallet_ethereum::Receipt>> {
			pallet_ethereum::CurrentReceipts::<Runtime>::get()
		}

		fn current_all() -> (
			Option<pallet_ethereum::Block>,
			Option<Vec<pallet_ethereum::Receipt>>,
			Option<Vec<TransactionStatus>>
		) {
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentReceipts::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get(),
			)
		}

		fn extrinsic_filter(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> Vec<EthereumTransaction> {
			xts.into_iter().filter_map(|xt| match xt.0.function {
				RuntimeCall::Ethereum(transact { transaction }) => Some(transaction),
				_ => None
			}).collect::<Vec<EthereumTransaction>>()
		}

		fn elasticity() -> Option<Permill> {
			Some(pallet_base_fee::Elasticity::<Runtime>::get())
		}

		fn gas_limit_multiplier_support() {}

		fn pending_block(
			xts: Vec<<Block as BlockT>::Extrinsic>,
		) -> (Option<pallet_ethereum::Block>, Option<Vec<TransactionStatus>>) {
			for ext in xts.into_iter() {
				let _ = Executive::apply_extrinsic(ext);
			}
			Ethereum::on_finalize(System::block_number() + 1);
			(
				pallet_ethereum::CurrentBlock::<Runtime>::get(),
				pallet_ethereum::CurrentTransactionStatuses::<Runtime>::get(),
			)
		}

		fn initialize_pending_block(header: &<Block as BlockT>::Header) {
			Executive::initialize_block(header);
		}
	}
	impl fp_rpc::ConvertTransactionRuntimeApi<Block> for Runtime {
		fn convert_transaction(transaction: EthereumTransaction) -> <Block as BlockT>::Extrinsic {
			UncheckedExtrinsic::new_unsigned(
				pallet_ethereum::Call::<Runtime>::transact { transaction }.into(),
			)
		}
	}
}
