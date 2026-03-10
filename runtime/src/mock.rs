//! # runtime/src/mock.rs
//!
//! XNET Runtime integration testi uchun yordamchi funksiyalar.
//! Haqiqiy node ishga tushirmasdan, runtime ni to'liq holda tekshiradi.

#![cfg(test)]

use crate::*;
use sp_keyring::AccountKeyring;
use sp_runtime::BuildStorage;

/// Test uchun Alice account ID si.
pub fn alice() -> AccountId {
	AccountKeyring::Alice.to_account_id()
}

/// Test uchun Bob account ID si.
pub fn bob() -> AccountId {
	AccountKeyring::Bob.to_account_id()
}

/// Treasury account ID si — PalletId dan hosil qilinadi.
pub fn treasury_account() -> AccountId {
	use sp_runtime::traits::AccountIdConversion;
	TreasuryPalletId::get().into_account_truncating()
}

/// Grant pool account ID si.
pub fn grant_account() -> AccountId {
	use sp_runtime::traits::AccountIdConversion;
	GrantPalletId::get().into_account_truncating()
}

/// Bo'sh genesis bilan TestExternalities yaratish.
pub fn new_test_ext() -> sp_io::TestExternalities {
	let storage = RuntimeGenesisConfig::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(storage);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

/// Alice va Bob ga boshlang'ich balans berib TestExternalities yaratish.
pub fn new_test_ext_with_balances(balances: Vec<(AccountId, Balance)>) -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Runtime>::default()
		.build_storage()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> { balances }
		.assimilate_storage(&mut t)
		.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}