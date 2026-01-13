use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GrandpaConfig, RuntimeGenesisConfig, Signature,
	SudoConfig, SystemConfig, WASM_BINARY, SessionConfig, StakingConfig, 
	opaque::SessionKeys, StakerStatus,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use serde_json::json;
use hex_literal::hex; // Hex kodni o'qish uchun maxsus kutubxona

// ============================================================================
// XnetXCoin Master Settings
// ============================================================================

/// Token sozlamalari: 1 XNX = 10^12 (12 decimals)
pub const XNX: u128 = 1_000_000_000_000;

/// Founder uchun 6 million XNX
pub const PREMINE: u128 = 6_000_000 * XNX;

/// Validator bo'lish uchun minimal balans
const VALIDATOR_STAKE: u128 = 10_000 * XNX;

/// Sizning hamyoningiz (Master Wallet)
/// Bu yerda hex format ishlatilmoqda
pub const FOUNDER_ACCOUNT: [u8; 32] = hex!("ca5344670f46bb69639065a18c1a21df152e14ec6a138e90fc1377bd5ffa4819");

/// Specialized `ChainSpec`.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

// --- Yordamchi Funksiyalar ---

/// Session keys generator
fn session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
	SessionKeys { aura, grandpa }
}

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate authority keys for a validator.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

// --- CONFIGURATION 1: DEVELOPMENT ---
pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"XnetXCoin Development",
		"xnx_dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account (Sizning hamyoningiz)
				FOUNDER_ACCOUNT.into(),
				// Pre-funded accounts
				vec![
					(FOUNDER_ACCOUNT.into(), PREMINE),
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 100_000 * XNX),
				],
				true,
			)
		},
		vec![],
		None,
		Some("xnx"),
		None,
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 12,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		None,
	))
}

// --- CONFIGURATION 2: LOCAL TESTNET ---
pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Local testnet wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"XnetXCoin Local Testnet",
		"xnx_local",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				vec![
					authority_keys_from_seed("Alice"),
					authority_keys_from_seed("Bob"),
				],
				FOUNDER_ACCOUNT.into(),
				vec![
					(FOUNDER_ACCOUNT.into(), PREMINE),
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 10_000 * XNX),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 10_000 * XNX),
				],
				true,
			)
		},
		vec![],
		None,
		Some("xnx"),
		None,
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 12,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		None,
	))
}

// --- CONFIGURATION 3: MAINNET (ENG MUHIMI) ---
pub fn mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"XnetXCoin Mainnet",
		"mainnet", // ID "mainnet" deb belgilandi
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm_binary,
				// Mainnetda hozircha 1 ta validator (keyin ko'paytiriladi)
				vec![authority_keys_from_seed("Validator1")],
				// Sudo (Boshqaruvchi) - Sizning hamyoningiz
				FOUNDER_ACCOUNT.into(),
				// Balanslar
				vec![
					// Sizning hamyoningizga 6 mln XNX tushadi
					(FOUNDER_ACCOUNT.into(), PREMINE),
				],
				true,
			)
		},
		vec![],
		None,
		Some("xnx"),
		None,
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 12,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		None,
	))
}

// --- GENESIS BUILDER ---
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
	_enable_println: bool,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			balances: endowed_accounts,
		},
		aura: AuraConfig {
			authorities: vec![], // Session orqali boshqariladi
		},
		grandpa: GrandpaConfig {
			authorities: vec![], // Session orqali boshqariladi
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(), // Stash
						x.0.clone(), // Controller (Stash bilan bir xil qilindi soddalik uchun)
						session_keys(x.2.clone(), x.3.clone()),
					)
				})
				.collect(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: 1,
			stakers: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.1.clone(),
						VALIDATOR_STAKE,
						StakerStatus::Validator,
					)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		sudo: SudoConfig {
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
	}
}
