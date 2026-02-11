use xnet_runtime::{
    AccountId, Signature, WASM_BINARY,
    opaque::SessionKeys, BABE_GENESIS_EPOCH_CONFIG,
};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::Perbill;
use serde_json::json;
use hex_literal::hex;

// --- TOKENOMICS CONSTANTS ---
pub const XNC: u128 = 1_000_000_000_000_000_000;
pub const PREMINE_AMOUNT: u128 = 6_000_000 * XNC;
const VALIDATOR_STAKE: u128 = 8_000 * XNC;
const NOMINATOR_STAKE: u128 = 1_000 * XNC;

// Founder Account
pub const FOUNDER_ACCOUNT_ID: [u8; 32] = hex!("563dc8be750abf3ad1e27ebf82e3b1eb6ab5a66aa7d3802660e07f8e72e27678");

pub type ChainSpec = sc_service::GenericChainSpec<sc_service::NoExtension>;

type AccountPublic = <Signature as Verify>::Signer;

/// Helper: Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper: Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Helper: Generate session keys (BABE + GRANDPA)
fn session_keys(babe: BabeId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { babe, grandpa }
}

/// Helper: Generate authority keys from seed
/// DIQQAT: Bu yerdan ImOnlineId olib tashlandi!
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s), // Stash
        get_account_id_from_seed::<sr25519::Public>(s), // Controller
        get_from_seed::<GrandpaId>(s),
        get_from_seed::<BabeId>(s),
    )
}

// --- CONFIG 1: DEVELOPMENT ---
pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("XNET Development")
    .with_id("dev")
    .with_chain_type(ChainType::Development)
    .with_properties(
        json!({
            "tokenSymbol": "XNC",
            "tokenDecimals": 18,
            "ss58Format": 42
        })
        .as_object()
        .expect("Properties must be a JSON object")
        .clone(),
    )
    .with_genesis_config_patch(testnet_genesis(
        // Initial Authorities (Alice only)
        vec![authority_keys_from_seed("Alice")],
        // Sudo Account (Founder)
        FOUNDER_ACCOUNT_ID.into(),
        // Pre-funded Accounts
        vec![
            (FOUNDER_ACCOUNT_ID.into(), PREMINE_AMOUNT),
            (get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000 * XNC),
        ],
    ))
    .build())
}

// --- CONFIG 2: LOCAL TESTNET (3 Nodes) ---
pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Local testnet wasm not available".to_string())?;

    Ok(ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("XNET Local Testnet")
    .with_id("local_testnet")
    .with_chain_type(ChainType::Local)
    .with_properties(
        json!({
            "tokenSymbol": "XNC",
            "tokenDecimals": 18,
            "ss58Format": 42
        })
        .as_object()
        .expect("Properties must be a JSON object")
        .clone(),
    )
    .with_genesis_config_patch(testnet_genesis(
        vec![
            authority_keys_from_seed("Alice"),
            authority_keys_from_seed("Bob"),
            authority_keys_from_seed("Charlie"),
        ],
        // Sudo Account
        FOUNDER_ACCOUNT_ID.into(),
        vec![
            (FOUNDER_ACCOUNT_ID.into(), PREMINE_AMOUNT),
            (get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000 * XNC),
            (get_account_id_from_seed::<sr25519::Public>("Bob"), 1_000_000 * XNC),
            (get_account_id_from_seed::<sr25519::Public>("Charlie"), 1_000_000 * XNC),
        ],
    ))
    .build())
}

// --- CONFIG 3: MAINNET ---
pub fn mainnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Mainnet wasm not available".to_string())?;

    Ok(ChainSpec::builder(
        wasm_binary,
        None,
    )
    .with_name("XNET Protocol Mainnet")
    .with_id("xnet_mainnet")
    .with_chain_type(ChainType::Live)
    .with_properties(
        json!({
            "tokenSymbol": "XNC",
            "tokenDecimals": 18,
            "ss58Format": 42
        })
        .as_object()
        .expect("Properties must be a JSON object")
        .clone(),
    )
    .with_genesis_config_patch(testnet_genesis(
        // Initial Validators (Placeholder: Alice)
        vec![authority_keys_from_seed("Alice")], 
        // Sudo Key
        FOUNDER_ACCOUNT_ID.into(),
        // Pre-mine
        vec![
            (FOUNDER_ACCOUNT_ID.into(), PREMINE_AMOUNT),
        ],
    ))
    .build())
}

// --- GENESIS BUILDER (JSON Format) ---
fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>, // DIQQAT: Bu yerda ham ImOnlineId olib tashlandi!
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, u128)>,
) -> serde_json::Value {
    
    serde_json::json!({
        "system": {},
        "balances": {
            "balances": endowed_accounts,
        },
        "babe": {
            "authorities": [],
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {
            "authorities": [],
        },
        "session": {
            "keys": initial_authorities.iter().map(|x| {
                (
                    x.0.clone(),
                    x.0.clone(), 
                    session_keys(x.3.clone(), x.2.clone()), // Faqat 2 ta argument
                )                                                                           
            }).collect::<Vec<_>>(),
        },
        "staking": {
            "validatorCount": initial_authorities.len() as u32,
            "minimumValidatorCount": 1,
            "invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            "slashRewardFraction": Perbill::from_percent(10),
            "minNominatorBond": NOMINATOR_STAKE,
            "minValidatorBond": VALIDATOR_STAKE,
            "stakers": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(), 
                        x.1.clone(), 
                        VALIDATOR_STAKE, 
                        "Validator", 
                    )
                })
                .collect::<Vec<_>>(),
        },
        "sudo": {
            "key": Some(root_key),
        },
        "treasury": {},
        "transactionPayment": {},
    })
}
