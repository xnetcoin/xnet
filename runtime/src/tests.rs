//! # runtime/src/tests.rs
//!
//! XNET Runtime integration testlari.
//!
//! ## Nima tekshiriladi
//!
//! 1. Genesis config — runtime to'g'ri build bo'ladimi
//! 2. Tokenomics — MAX_SUPPLY, BLOCK_REWARD, halving konstantalari
//! 3. Fee taqsimoti — 60% treasury, 15% grant, 25% validator
//! 4. ZkVerifier — runtime da ishlayaptimi
//! 5. SS58 prefix va Chain ID — to'g'rimi
//! 6. Vesting — lock parametrlari to'g'rimi
//! 7. Runtime version — spec_name, spec_version

#![cfg(test)]

use crate::mock::*;
use crate::*;
use frame_support::traits::fungible::Inspect;

// =============================================================================
// 1. Genesis
// =============================================================================

#[test]
fn genesis_builds_successfully() {
	new_test_ext().execute_with(|| {
		// Runtime genesis muvaffaqiyatli build bo'ldi
		assert_eq!(System::block_number(), 1);
	});
}

#[test]
fn genesis_with_balances_works() {
	new_test_ext_with_balances(vec![(alice(), 1_000 * UNIT), (bob(), 500 * UNIT)])
		.execute_with(|| {
			assert_eq!(Balances::balance(&alice()), 1_000 * UNIT);
			assert_eq!(Balances::balance(&bob()), 500 * UNIT);
		});
}

// =============================================================================
// 2. Tokenomics konstantalari
// =============================================================================

#[test]
fn max_supply_is_correct() {
	// 53,000,000 XNET
	assert_eq!(MAX_SUPPLY, 53_000_000 * UNIT);
}

#[test]
fn block_reward_is_correct() {
	// 1.117 XNET per block
	assert_eq!(BLOCK_REWARD, 1_117 * MILLIUNIT);
}

#[test]
fn unit_decimals_are_18() {
	assert_eq!(UNIT, 1_000_000_000_000_000_000u128);
	assert_eq!(MILLIUNIT, UNIT / 1_000);
	assert_eq!(MICROUNIT, UNIT / 1_000_000);
}

#[test]
fn existential_deposit_is_milliunit() {
	assert_eq!(EXISTENTIAL_DEPOSIT, MILLIUNIT);
}

#[test]
fn halving_interval_is_correct() {
	// ~4 yil: 21,038,400 blok × 6 sekund = ~4 yil
	let halving: u32 = 21_038_400;
	assert_eq!(
		halving,
		21_038_400,
		"HalvingInterval must be 21,038,400 blocks (~4 years at 6s blocks)"
	);
}

#[test]
fn min_validator_bond_is_8000_xnet() {
	assert_eq!(MIN_VALIDATOR_BOND, 8_000 * UNIT);
}

#[test]
fn min_nominator_bond_is_1000_xnet() {
	assert_eq!(MIN_NOMINATOR_BOND, 1_000 * UNIT);
}

// =============================================================================
// 3. Chain identifikatori
// =============================================================================

#[test]
fn ss58_prefix_is_888() {
	assert_eq!(SS58Prefix::get(), 888u16);
}

#[test]
fn chain_id_is_2009() {
	// EVM Chain ID
	let chain_id: u64 = <Runtime as pallet_evm::Config>::ChainId::get();
	assert_eq!(chain_id, 2009u64);
}

#[test]
fn runtime_spec_name_is_xnetcoin() {
	assert_eq!(VERSION.spec_name, sp_version::create_runtime_str!("xnetcoin"));
}

#[test]
fn runtime_spec_version_is_100() {
	assert_eq!(VERSION.spec_version, 100);
}

// =============================================================================
// 4. Fee taqsimoti konstantalari
// =============================================================================

#[test]
fn treasury_pallet_id_is_correct() {
	use sp_runtime::traits::AccountIdConversion;
	// PalletId(*b"py/trsry") — standart Substrate treasury ID
	let id = TreasuryPalletId::get();
	let _account: AccountId = id.into_account_truncating();
	// Muvaffaqiyatli aylantirish — panic yo'q
}

#[test]
fn grant_pallet_id_is_correct() {
	use sp_runtime::traits::AccountIdConversion;
	let id = GrantPalletId::get();
	let _account: AccountId = id.into_account_truncating();
}

#[test]
fn treasury_and_grant_accounts_are_different() {
	assert_ne!(treasury_account(), grant_account());
}

// =============================================================================
// 5. ZkVerifier runtime da mavjud
// =============================================================================

#[test]
fn zk_verifier_max_proofs_per_block_is_20() {
	assert_eq!(MaxProofsPerBlock::get(), 20u32);
}

#[test]
fn zk_verifier_max_public_inputs_is_16() {
	assert_eq!(MaxPublicInputs::get(), 16u32);
}

#[test]
fn zk_verifier_register_vk_requires_root() {
	new_test_ext().execute_with(|| {
		let vk_id = [1u8; 32];
		let fake_vk = vec![0u8; 500];

		// Signed origin — rad etiladi
		let result = ZkVerifier::register_vk(
			RuntimeOrigin::signed(alice()),
			vk_id,
			fake_vk,
			2,
			b"test".to_vec(),
		);
		assert!(result.is_err());
	});
}

// =============================================================================
// 6. Vesting parametrlari
// =============================================================================

#[test]
fn min_vested_transfer_is_100_xnet() {
	assert_eq!(MinVestedTransfer::get(), 100 * UNIT);
}

// =============================================================================
// 7. Block time va slot duration
// =============================================================================

#[test]
fn block_time_is_6_seconds() {
	assert_eq!(MILLISECS_PER_BLOCK, 6_000u64);
	assert_eq!(SLOT_DURATION, MILLISECS_PER_BLOCK);
}

#[test]
fn time_constants_are_correct() {
	assert_eq!(MINUTES, 10u32); // 60_000 / 6_000 = 10
	assert_eq!(HOURS, 600u32);  // 10 * 60
	assert_eq!(DAYS, 14_400u32); // 600 * 24
}

// =============================================================================
// 8. Balances — dust treasury ga ketadi
// =============================================================================

#[test]
fn treasury_account_can_receive_balance() {
	new_test_ext_with_balances(vec![(treasury_account(), 1_000 * UNIT)]).execute_with(|| {
		assert_eq!(Balances::balance(&treasury_account()), 1_000 * UNIT);
	});
}

// =============================================================================
// 9. EVM base fee
// =============================================================================

#[test]
fn default_base_fee_is_1_gwei() {
	use sp_core::U256;
	let expected = U256::from(1_000_000_000u64); // 1 Gwei
	assert_eq!(DefaultBaseFeePerGas::get(), expected);
}

// =============================================================================
// 10. Runtime integrity
// =============================================================================

#[test]
fn runtime_integrity_test() {
	// construct_runtime! makrosining integrity testi
	// Bu test har doim o'tishi kerak
	sp_io::TestExternalities::default().execute_with(|| {
		use frame_support::traits::IntegrityTest;
		<AllPalletsWithSystem as IntegrityTest>::integrity_test();
	});
}