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
    traits::{ConstU32, ConstU64, Everything},
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
    type BlockHashCount         = BlockHashCount;
    type Version                = ();
    type PalletInfo             = PalletInfo;
    type AccountData            = ();
    type OnNewAccount           = ();
    type OnKilledAccount        = ();
    type SystemWeightInfo       = ();
    type SS58Prefix             = ConstU32<888>;
    type OnSetCode              = ();
    type MaxConsumers           = ConstU32<16>;
    type RuntimeTask            = ();
    type SingleBlockMigrations  = ();
    type MultiBlockMigrator     = ();
    type PreInherents           = ();
    type PostInherents          = ();
    type PostTransactions       = ();
}

// ─────────────────────────────────────────────────────────────────────────────
// pallet-zk-verifier konfiguratsiyasi
// ─────────────────────────────────────────────────────────────────────────────

parameter_types! {
    pub const MaxProofsPerBlock: u32 = 10;
    pub const MaxPublicInputs:   u32 = 16;
}

impl pallet_zk_verifier::Config for Test {
    type RuntimeEvent     = RuntimeEvent;
    type WeightInfo       = crate::weights::TestWeightInfo;
    type MaxProofsPerBlock = MaxProofsPerBlock;
    type MaxPublicInputs  = MaxPublicInputs;
}

// ─────────────────────────────────────────────────────────────────────────────
// Test yordamchi funksiyalari
// ─────────────────────────────────────────────────────────────────────────────

/// Bo'sh holat bilan test muhitini ishga tushirish.
pub fn new_test_ext() -> sp_io::TestExternalities {
    let storage = frame_system::GenesisConfig::<Test>::default()
        .build_storage()
        .unwrap();
    sp_io::TestExternalities::new(storage)
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

/// Test uchun G1 nuqta (BN254 generator).
/// x = 1, y = 2 — bu BN254'ning haqiqiy generator nuqtasi.
pub fn g1_generator() -> Vec<u8> {
    let mut g1 = vec![0u8; 64];
    g1[31] = 1; // x = 1
    g1[63] = 2; // y = 2
    g1
}

/// Test uchun G2 nuqta (BN254 generator).
pub fn g2_generator() -> Vec<u8> {
    // BN254 G2 generator koordinatalari (big-endian)
    let x_re: [u8; 32] = [
        0x19, 0x8e, 0x93, 0x93, 0x92, 0x0d, 0x48, 0x3a,
        0x70, 0x26, 0x33, 0xcf, 0xeb, 0xbc, 0xf9, 0x28,
        0x41, 0x54, 0x75, 0x19, 0x72, 0xd1, 0xf7, 0xf0,
        0x31, 0x00, 0xa7, 0x88, 0x55, 0x33, 0x16, 0x89,
    ];
    let x_im: [u8; 32] = [
        0x09, 0x89, 0xba, 0x96, 0x14, 0x99, 0x2c, 0xac,
        0x62, 0x70, 0x64, 0x04, 0x4c, 0x5e, 0x3c, 0xa4,
        0xda, 0x5c, 0xa3, 0x28, 0xaf, 0xa7, 0x48, 0x12,
        0x28, 0x30, 0xbb, 0xc5, 0x26, 0xf2, 0xcd, 0x25,
    ];
    let y_re: [u8; 32] = [
        0x09, 0x44, 0x40, 0xf7, 0xef, 0xab, 0xf0, 0xa3,
        0x49, 0x37, 0x44, 0x9d, 0x5c, 0xc2, 0x13, 0x0e,
        0x32, 0x86, 0x98, 0xe7, 0xf8, 0xe0, 0x38, 0x40,
        0x36, 0xf5, 0x5a, 0x4c, 0xb0, 0xfb, 0xb1, 0x55,
    ];
    let y_im: [u8; 32] = [
        0x09, 0x89, 0xba, 0x96, 0x14, 0x99, 0x2c, 0xac,
        0x62, 0x70, 0x64, 0x04, 0x4c, 0x5e, 0x3c, 0xa4,
        0xda, 0x5c, 0xa3, 0x28, 0xaf, 0xa7, 0x48, 0x12,
        0x28, 0x30, 0xbb, 0xc5, 0x26, 0xf2, 0xcd, 0x26,
    ];
    let mut g2 = Vec::with_capacity(128);
    g2.extend_from_slice(&x_re);
    g2.extend_from_slice(&x_im);
    g2.extend_from_slice(&y_re);
    g2.extend_from_slice(&y_im);
    g2
}

/// Minimal to'g'ri VK yaratish (1 ta public input uchun).
/// ic_len = 2: ic[0] + 1 ta input uchun ic[1]
pub fn make_test_vk() -> (Vec<u8>, u32) {
    let g1 = g1_generator();
    let g2 = g2_generator();

    let mut vk = Vec::new();
    vk.extend_from_slice(&g1); // alpha_g1 (64)
    vk.extend_from_slice(&g2); // beta_g2  (128)
    vk.extend_from_slice(&g2); // gamma_g2 (128)
    vk.extend_from_slice(&g2); // delta_g2 (128)
    vk.extend_from_slice(&g1); // ic[0]    (64)
    vk.extend_from_slice(&g1); // ic[1]    (64)
    // Jami: 64 + 128 + 128 + 128 + 64 + 64 = 576 bayt

    let ic_len = 2u32; // 1 ta public input + 1
    (vk, ic_len)
}

/// 256 bayt noldan iborat soxta proof.
pub fn zero_proof() -> [u8; 256] {
    [0u8; 256]
}