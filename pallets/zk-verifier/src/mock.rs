//! # mock.rs
//!
//! `tests.rs` uchun miniatyura XNET tarmog'i.
//!
//! Haqiqiy tarmoqni ishga tushirmasdan, pallet'ni izolyatsiyada
//! tekshirish imkonini beradi. `frame_support::construct_runtime!`
//! faqat kerakli pallet'larni yig'adi.

#![cfg(test)]

use frame_support::{
	construct_runtime, parameter_types,
	traits::{ConstU32, ConstU64, ConstU16, Everything},
};
use sp_core::H256;
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

use crate as pallet_zk_verifier;

// ─────────────────────────────────────────────────────────────────────────────
// Test runtime
// ─────────────────────────────────────────────────────────────────────────────

type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test {
		System:    frame_system,
		ZkVerifier: pallet_zk_verifier,
	}
);

// ─────────────────────────────────────────────────────────────────────────────
// frame_system konfiguratsiyasi
// ─────────────────────────────────────────────────────────────────────────────

parameter_types! {
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type BaseCallFilter = Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = BlockHashCount;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix             = ConstU16<88>;
	type OnSetCode = ();
	type MaxConsumers = ConstU32<16>;
	type RuntimeTask = ();
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

// ─────────────────────────────────────────────────────────────────────────────
// pallet-zk-verifier konfiguratsiyasi
// ─────────────────────────────────────────────────────────────────────────────

parameter_types! {
	pub const MaxProofsPerBlock: u32 = 10;
	pub const MaxPublicInputs:   u32 = 16;
}

impl pallet_zk_verifier::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::TestWeightInfo;
	type MaxProofsPerBlock = MaxProofsPerBlock;
	type MaxPublicInputs = MaxPublicInputs;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test yordamchi funksiyalari
// ─────────────────────────────────────────────────────────────────────────────

/// Bo'sh holat bilan test muhitini ishga tushirish.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(storage);
	// Block number must be > 0 for System::assert_has_event to find events.
	ext.execute_with(|| System::set_block_number(1));
	ext
}

/// Test uchun VK ID — qisqa yozuv.
pub fn vk_id(tag: &[u8]) -> [u8; 32] {
	let mut id = [0u8; 32];
	let len = tag.len().min(32);
	id[..len].copy_from_slice(&tag[..len]);
	id
}

/// Test uchun nullifier — qisqa yozuv.
pub fn nullifier(n: u8) -> [u8; 32] {
	let mut null = [0u8; 32];
	null[31] = n;
	null
}

/// Builds a minimal but cryptographically valid `VerifyingKey<Bn254>` for tests
/// and returns its `ark-serialize` compressed bytes together with ic_len = 2
/// (one public input).
///
/// All curve points are set to the BN254 generator (G::generator()), which
/// produces a valid point on the curve. The resulting VerifyingKey won't satisfy
/// the Groth16 equation for any real proof, but it will deserialize successfully
/// inside `register_vk`.
pub fn make_test_vk() -> (Vec<u8>, u32) {
	use ark_bn254::{Bn254, G1Affine, G2Affine};
	use ark_ec::AffineRepr;
	use ark_groth16::VerifyingKey;
	use ark_serialize::CanonicalSerialize;

	let g1 = G1Affine::generator();
	let g2 = G2Affine::generator();

	let vk: VerifyingKey<Bn254> = VerifyingKey {
		alpha_g1: g1,
		beta_g2:  g2,
		gamma_g2: g2,
		delta_g2: g2,
		// ic must have ic_len elements; we use 2 (one public input)
		gamma_abc_g1: vec![g1, g1],
	};

	let mut vk_bytes = Vec::new();
	vk.serialize_compressed(&mut vk_bytes)
		.expect("VerifyingKey serialization must not fail in tests");

	// ic_len = gamma_abc_g1.len() = 2  →  1 public input
	(vk_bytes, 2u32)
}

/// Returns the BN254 G1 generator point in uncompressed big-endian format
/// (x: 32 bytes, y: 32 bytes) for tests that check coordinate values directly.
pub fn g1_generator() -> Vec<u8> {
	let mut g1 = vec![0u8; 64];
	g1[31] = 1; // x = 1 (big-endian)
	g1[63] = 2; // y = 2 (big-endian)
	g1
}

/// Returns a zeroed-out 256-byte fake proof.
///
/// This will always fail the Groth16 pairing check, which is the intended
/// behaviour for `zero_proof_is_rejected_by_math` and related tests.
pub fn zero_proof() -> [u8; 256] {
	[0u8; 256]
}

