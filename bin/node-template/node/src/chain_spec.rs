use node_template_runtime::{
	AccountId, AuraConfig, BalancesConfig, GrandpaConfig, RuntimeGenesisConfig, Signature,
	SudoConfig, SystemConfig, SessionConfig, StakingConfig, WASM_BINARY, PREMINE,
	opaque::SessionKeys, StakerStatus, session_keys,
};
use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public, crypto::Ss58Codec};
use sp_runtime::traits::{IdentifyAccount, Verify};
use serde_json::json;

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

// ============================================================================
// XnetXCoin Mainnet Wallet Addresses
// ============================================================================

/// Founder/Premine wallet address - 6,000,000 XNX
const FOUNDER_ADDRESS: &str = "5G1YRg4aKtHemtpuxh15u3YCBJwYBWt8ykeGbqhxDGuJTNXQ";

/// 1 XNX with 18 decimals
const XNX: u128 = 1_000_000_000_000_000_000;

/// Initial stake for validators (10,000 XNX)
const VALIDATOR_STAKE: u128 = 10_000 * XNX;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

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
/// Returns (stash_account, controller_account, aura_key, grandpa_key)
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", s)),
		get_account_id_from_seed::<sr25519::Public>(s),
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"XnetXCoin Development",
		// ID
		"xnx_dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts with XNX balances
				vec![
					// Alice (premine for dev)
					(get_account_id_from_seed::<sr25519::Public>("Alice"), PREMINE),
					// Alice stash (for staking)
					(get_account_id_from_seed::<sr25519::Public>("Alice//stash"), VALIDATOR_STAKE * 2),
					// Test accounts
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 100_000 * XNX),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 10_000 * XNX),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("xnx"),
		None,
		// Properties
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 18,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		// Extensions
		None,
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"XnetXCoin Testnet",
		// ID
		"xnx_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm_binary,
				// Initial PoA authorities
				vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts with XNX balances
				vec![
					// Premine wallet
					(get_account_id_from_seed::<sr25519::Public>("Alice"), PREMINE),
					// Validator stash accounts
					(get_account_id_from_seed::<sr25519::Public>("Alice//stash"), VALIDATOR_STAKE * 2),
					(get_account_id_from_seed::<sr25519::Public>("Bob//stash"), VALIDATOR_STAKE * 2),
					// Controller accounts (need some balance for fees)
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 1_000 * XNX),
					// Additional test accounts
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 50_000 * XNX),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 50_000 * XNX),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("xnx"),
		// Properties
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 18,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		None,
		// Extensions
		None,
	))
}

/// Helper function to convert SS58 address to AccountId
fn account_id_from_ss58(address: &str) -> AccountId {
	AccountId::from_ss58check(address).expect("Invalid SS58 address")
}

/// XnetXCoin Mainnet configuration
pub fn mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"XnetXCoin Mainnet",
		// ID
		"xnx_mainnet",
		ChainType::Live,
		move || {
			mainnet_genesis(
				wasm_binary,
				// Initial validators - TODO: Replace with real validator keys before mainnet launch
				vec![authority_keys_from_seed("Validator1")],
				// Sudo account - founder
				account_id_from_ss58(FOUNDER_ADDRESS),
				// Premine allocation
				vec![
					// Founder wallet - 6,000,000 XNX
					(account_id_from_ss58(FOUNDER_ADDRESS), PREMINE),
					// Initial validator stash - TODO: Replace with real validator addresses
					(get_account_id_from_seed::<sr25519::Public>("Validator1//stash"), VALIDATOR_STAKE * 2),
				],
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		Some("xnx"),
		// Fork ID
		None,
		// Properties
		Some(json!({
			"tokenSymbol": "XNX",
			"tokenDecimals": 18,
			"ss58Format": 42
		}).as_object().unwrap().clone()),
		// Extensions
		None,
	))
}

/// Configure mainnet genesis state
fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, u128)>,
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
			authorities: vec![],  // Managed by Session pallet
		},
		grandpa: GrandpaConfig {
			authorities: vec![],  // Managed by Session pallet
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|(stash, _, aura, grandpa)| {
					(
						stash.clone(),
						stash.clone(),
						session_keys(aura.clone(), grandpa.clone()),
					)
				})
				.collect(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: 1,
			stakers: initial_authorities
				.iter()
				.map(|(stash, controller, _, _)| {
					(stash.clone(), controller.clone(), VALIDATOR_STAKE, StakerStatus::Validator)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|(s, _, _, _)| s.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		sudo: SudoConfig {
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
	}
}

/// Configure initial storage state for FRAME modules.
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
			authorities: vec![],  // Managed by Session pallet
		},
		grandpa: GrandpaConfig {
			authorities: vec![],  // Managed by Session pallet
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|(stash, _, aura, grandpa)| {
					(
						stash.clone(),
						stash.clone(),
						session_keys(aura.clone(), grandpa.clone()),
					)
				})
				.collect(),
		},
		staking: StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: 1,
			stakers: initial_authorities
				.iter()
				.map(|(stash, controller, _, _)| {
					(stash.clone(), controller.clone(), VALIDATOR_STAKE, StakerStatus::Validator)
				})
				.collect(),
			invulnerables: initial_authorities.iter().map(|(s, _, _, _)| s.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		},
		sudo: SudoConfig {
			key: Some(root_key),
		},
		transaction_payment: Default::default(),
	}
}
