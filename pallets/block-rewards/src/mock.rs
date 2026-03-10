//! # Block Reward Pallet — Test Runtime
//!
//! Provides a minimal `construct_runtime!` environment for unit-testing
//! `pallet-block-reward` in isolation. Key design decisions:
//!
//! - `FindAuthor` is a stub that always returns account `ALICE` (id = 1).
//!   Real BABE pre-runtime digests would be required in integration tests,
//!   but unit tests only care about the reward logic, not consensus.
//!
//! - `HalvingInterval` is set to 10 blocks so halving behaviour can be
//!   exercised without running hundreds of thousands of blocks.
//!
//! - `InitialBlockReward` is 1_000 planck, keeping arithmetic readable.
//!
//! - `MaxSupply` is 10_000 planck, small enough to reach in tens of blocks.

#![cfg(test)]

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU16, ConstU32, ConstU64, Everything, FindAuthor, Hooks},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	ConsensusEngineId, BuildStorage,
};

use crate as pallet_block_reward;

// =============================================================================
// Block author constants
// =============================================================================

/// The account id that the mock FindAuthor always returns.
pub const ALICE: u64 = 1;
/// A second account used for pre-funding checks.
pub const BOB: u64 = 2;

// =============================================================================
// Test runtime
// =============================================================================

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System:      frame_system,
		Balances:    pallet_balances,
		BlockReward: pallet_block_reward,
	}
);

// =============================================================================
// frame_system
// =============================================================================

impl frame_system::Config for Test {
	type BaseCallFilter         = Everything;
	type BlockWeights           = ();
	type BlockLength            = ();
	type DbWeight               = ();
	type RuntimeOrigin          = RuntimeOrigin;
	type RuntimeCall            = RuntimeCall;
	type Nonce                  = u64;
	type Hash                   = H256;
	type Hashing                = BlakeTwo256;
	type AccountId              = u64;
	type Lookup                 = IdentityLookup<Self::AccountId>;
	type Block                  = Block;
	type RuntimeEvent           = RuntimeEvent;
	type BlockHashCount         = ConstU64<250>;
	type Version                = ();
	type PalletInfo             = PalletInfo;
	type AccountData            = pallet_balances::AccountData<u64>;
	type OnNewAccount           = ();
	type OnKilledAccount        = ();
	type SystemWeightInfo       = ();
	type SS58Prefix             = ConstU16<88>;
	type OnSetCode              = ();
	type MaxConsumers           = ConstU32<16>;
	type RuntimeTask            = ();
	type SingleBlockMigrations  = ();
	type MultiBlockMigrator     = ();
	type PreInherents           = ();
	type PostInherents          = ();
	type PostTransactions       = ();
}

// =============================================================================
// pallet_balances
// =============================================================================

parameter_types! {
	pub const MaxLocks: u32 = 50;
}

impl pallet_balances::Config for Test {
	type MaxLocks           = MaxLocks;
	type MaxReserves        = ();
	type ReserveIdentifier  = [u8; 8];
	type Balance            = u64;
	type RuntimeEvent       = RuntimeEvent;
	type DustRemoval        = ();
	type ExistentialDeposit = ConstU64<1>;
	type AccountStore       = System;
	type WeightInfo         = ();
	type FreezeIdentifier   = ();
	type MaxFreezes         = ();
	type RuntimeHoldReason  = ();
	type RuntimeFreezeReason = ();
}

// =============================================================================
// FindAuthor stub — always returns ALICE
// =============================================================================

/// A `FindAuthor` implementation that ignores the digest and unconditionally
/// returns `ALICE`. This lets reward tests run without injecting BABE digests.
pub struct AlwaysAlice;
impl FindAuthor<u64> for AlwaysAlice {
	fn find_author<'a, I>(_digests: I) -> Option<u64>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		Some(ALICE)
	}
}

/// A `FindAuthor` stub that never resolves an author.
/// Used to test the code-path where no block author is found.
pub struct NoAuthor;
impl FindAuthor<u64> for NoAuthor {
	fn find_author<'a, I>(_digests: I) -> Option<u64>
	where
		I: 'a + IntoIterator<Item = (ConsensusEngineId, &'a [u8])>,
	{
		None
	}
}

// =============================================================================
// pallet_block_reward parameters
// =============================================================================

parameter_types! {
	/// Starting reward per block before any halvings — kept small for tests.
	pub const InitialBlockReward: u64 = 1_000;

	/// Halving every 10 blocks so tests can exercise multiple periods quickly.
	pub const HalvingInterval: u64 = 10;

	/// Hard cap on total issuance — set low enough to reach within test time.
	pub const MaxSupply: u64 = 10_000;
}

impl pallet_block_reward::Config for Test {
	type RuntimeEvent       = RuntimeEvent;
	type Currency           = Balances;
	type FindAuthor         = AlwaysAlice;
	type InitialBlockReward = InitialBlockReward;
	type HalvingInterval    = HalvingInterval;
	type MaxSupply          = MaxSupply;
}

// =============================================================================
// Test helpers
// =============================================================================

/// Builds a clean `TestExternalities` with zero initial balances.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();
	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

/// Builds a `TestExternalities` with `ALICE` pre-funded to `amount`.
pub fn new_test_ext_with_balance(amount: u64) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Test> {
		balances: vec![(ALICE, amount)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

/// Advances the block number to `n` and triggers `on_finalize`.
/// Mirrors what the `Executive` does at the end of every real block.
pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		BlockReward::on_finalize(System::block_number());
		System::set_block_number(System::block_number() + 1);
	}
}

/// Runs a single `on_finalize` at the current block number, then increments.
pub fn finalize_block() {
	BlockReward::on_finalize(System::block_number());
	System::set_block_number(System::block_number() + 1);
}
