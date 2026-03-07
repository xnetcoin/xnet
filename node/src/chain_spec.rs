//! # Chain Specification
//!
//! Defines how the XNET blockchain is initialised at genesis for each deployment
//! environment: development (single-node Alice), local testnet (three nodes), and
//! mainnet. Each config sets the token name, decimal precision, SS58 prefix, and
//! the initial account balances, validator set, and sudo key.
//!
//! ## Tokenomics at Genesis
//! | Constant            | Value          | Notes                                 |
//! |---------------------|----------------|---------------------------------------|
//! | `PREMINE_AMOUNT`    | 6,000,000 XNC  | Allocated to the founder account      |
//! | `VALIDATOR_STAKE`   | 8,000 XNC      | Self-bond required per validator      |
//! | `NOMINATOR_STAKE`   | 1,000 XNC      | Minimum nominator bond                |

use xnet_runtime::{
    AccountId, Signature, WASM_BINARY,
    opaque::SessionKeys, BABE_GENESIS_EPOCH_CONFIG,
};
use sc_service::ChainType;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::Perbill;
use serde_json::json;
use hex_literal::hex;

// =============================================================================
// Genesis Tokenomics
// =============================================================================

/// One XNC token (18 decimal places).
pub const XNC: u128 = 1_000_000_000_000_000_000;

/// Tokens allocated to the founder address at genesis.
pub const PREMINE_AMOUNT: u128 = 6_000_000 * XNC;

/// Minimum self-bond required for a validator (matches `MIN_VALIDATOR_BOND` in runtime).
const VALIDATOR_STAKE: u128 = 8_000 * XNC;

/// Minimum nominator bond (matches `MIN_NOMINATOR_BOND` in runtime).
const NOMINATOR_STAKE: u128 = 1_000 * XNC;

/// SR25519 public key of the founder / initial sudo key.
/// Replace with the real key before mainnet launch.
pub const FOUNDER_ACCOUNT_ID: [u8; 32] = hex!("ca5344670f46bb69639065a18c1a21df152e14ec6a138e90fc1377bd5ffa4819");

/// Concrete chain-spec type — no custom extensions required.
pub type ChainSpec = sc_service::GenericChainSpec<sc_service::NoExtension>;

type AccountPublic = <Signature as Verify>::Signer;

/// Derives an `AccountId` from a development seed string (e.g. `"Alice"`).
///
/// Only use this for dev/testnet configs — seeds are not secure in production.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Derives a public key from a development seed string.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Packs BABE, GRANDPA, and ImOnline public keys into a single `SessionKeys` struct.
fn session_keys(babe: BabeId, grandpa: GrandpaId, im_online: ImOnlineId) -> SessionKeys {
    SessionKeys { babe, grandpa, im_online }
}

/// Derives a full set of authority keys (stash, controller, GRANDPA, BABE, ImOnline)
/// from a single development seed string.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s), // Stash    (x.0)
        get_account_id_from_seed::<sr25519::Public>(s), // Controller (x.1) — same as stash for dev
        get_from_seed::<GrandpaId>(s),                  // GRANDPA  (x.2)
        get_from_seed::<BabeId>(s),                     // BABE     (x.3)
        get_from_seed::<ImOnlineId>(s),                 // ImOnline (x.4)
    )
}

// =============================================================================
// Chain Configurations
// =============================================================================

/// Single-node development chain (instant sealing via Alice).
///
/// Useful for rapid local development — no need to run multiple nodes.
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
        // Single authority — Alice is both proposer and finaliser.
        vec![authority_keys_from_seed("Alice")],
        // Sudo key — founder address controls privileged calls during development.
        FOUNDER_ACCOUNT_ID.into(),
        // Initial balances.
        vec![
            (FOUNDER_ACCOUNT_ID.into(), PREMINE_AMOUNT),
            (get_account_id_from_seed::<sr25519::Public>("Alice"), 1_000_000 * XNC),
        ],
    ))
    .build())
}

/// Three-node local testnet — Alice, Bob, and Charlie share block production.
///
/// Use this to test validator rotation, slashing, and governance locally
/// before deploying to a public testnet.
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

/// Production mainnet configuration.
///
/// Before launch, replace the Alice placeholder authority with real validator keys
/// and update `FOUNDER_ACCOUNT_ID` to the actual genesis sudo key.
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
        // TODO: replace with real validator keys before mainnet launch.
        vec![authority_keys_from_seed("Alice")],
        FOUNDER_ACCOUNT_ID.into(),
        vec![
            (FOUNDER_ACCOUNT_ID.into(), PREMINE_AMOUNT),
        ],
    ))
    .build())
}

// =============================================================================
// Genesis Builder
// =============================================================================

/// Constructs the genesis storage patch shared by all chain configurations.
///
/// This function is the single source of truth for the initial chain state:
/// it wires together starting balances, the validator set, session keys,
/// staking configuration, and the sudo key.
fn testnet_genesis(
    initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId, ImOnlineId)>,
    root_key: AccountId,
    endowed_accounts: Vec<(AccountId, u128)>,
) -> serde_json::Value {

    serde_json::json!({
        "system": {},
        "balances": {
            // Accounts funded at genesis — includes premine and dev seeding.
            "balances": endowed_accounts,
        },
        "babe": {
            // Validators register their session keys via the `Session` pallet;
            // BABE authorities start empty and are populated at the first epoch.
            "authorities": [],
            "epochConfig": Some(BABE_GENESIS_EPOCH_CONFIG),
        },
        "grandpa": {
            // GRANDPA authority set is also managed dynamically via `Session`.
            "authorities": [],
        },
        "session": {
            // Map each validator's stash to their session keys (BABE + GRANDPA + ImOnline).
            "keys": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(), // Stash account
                        x.0.clone(), // Controller account (same as stash for dev)
                        session_keys(x.3.clone(), x.2.clone(), x.4.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        "staking": {
            "validatorCount": initial_authorities.len() as u32,
            "minimumValidatorCount": 1,
            // TESTNET: Invulnerable validators cannot be slashed. This is intentional
            // during the testnet phase — removing invulnerability now could cause the
            // network to halt if validators are slashed due to bugs or misconfiguration.
            // Remove this before mainnet launch so the NPoS security model is fully active.
            "invulnerables": initial_authorities.iter().map(|x| x.0.clone()).collect::<Vec<_>>(),
            // 10% of any slash goes to the reporter; the rest goes to the treasury.
            "slashRewardFraction": Perbill::from_percent(10),
            "minNominatorBond": NOMINATOR_STAKE,
            "minValidatorBond": VALIDATOR_STAKE,
            // Bootstrap stakers — each validator self-nominates at genesis.
            "stakers": initial_authorities
                .iter()
                .map(|x| {
                    (
                        x.0.clone(),       // Stash
                        x.1.clone(),       // Controller
                        VALIDATOR_STAKE,   // Initial self-bond
                        "Validator",       // Role
                    )
                })
                .collect::<Vec<_>>(),
        },
        "sudo": {
            // The sudo key can execute privileged calls until on-chain governance is live.
            "key": Some(root_key),
        },
        "treasury": {},
        "evm": {
            "accounts": {
                "0xaaafB3972B05630fCceE866eC69CdADd9baC2771": {
                    "balance": "0x56BC75E2D630FFFFF",
                    "code": [],
                    "nonce": "0x0",
                    "storage": {}
                },
                "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac": {
                    "balance": "0x56BC75E2D630FFFFF",
                    "code": [],
                    "nonce": "0x0",
                    "storage": {}
                }
            }
        },
        "ethereum": {},
        "baseFee": {
            "baseFeePerGas": "0x3B9ACA00",
            "elasticity": 125000
        },
        "transactionPayment": {
            "multiplier": "1000000000000000000"
        },
            // Vesting schedule:
            // (Who, Start Block, Length in Blocks, Locked Amount)
            // 6_000_000 XNC locked for 2.5 years (13,140,000 blocks at 6s/block).
        "vesting": {
            "vesting": vec![
                (
                    <AccountId as From<[u8; 32]>>::from(FOUNDER_ACCOUNT_ID),
                    0u32,                 // 0-blokdan boshlanadi
                    13_140_000u32,        // 2.5 yil davom etadi
                    5_500_000_000_000_000_000_000_000u128 // 5.5 million XNC (18 ta nol va oxirida u128)
                ),
            ],
        },
    })
}
