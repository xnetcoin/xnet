//! # XNET EVM Precompiles
//!
//! Implements the standard Ethereum precompile set (addresses `0x01`–`0x09`) for the
//! XNET EVM layer. Without these precompiles, any Solidity contract that calls a
//! standard cryptographic operation — elliptic curve maths, hash functions, modular
//! exponentiation — will revert or return unexpected results.
//!
//! ## Address Map
//!
//! | Address | Name            | Used by                                      |
//! |---------|-----------------|----------------------------------------------|
//! | `0x01`  | ECRecover       | Signature verification (MetaMask, wallets)   |
//! | `0x02`  | SHA-256         | Bitcoin bridge, general hashing              |
//! | `0x03`  | RIPEMD-160      | Bitcoin address derivation                   |
//! | `0x04`  | Identity        | Data copy — used internally by many contracts|
//! | `0x05`  | ModExp          | RSA, big-number crypto, some ZK circuits     |
//! | `0x06`  | BN128Add        | ZK-SNARK verification (Groth16, Plonk)       |
//! | `0x07`  | BN128Mul        | ZK-SNARK verification                        |
//! | `0x08`  | BN128Pairing    | ZK-SNARK verification — Uniswap v3 pricing   |
//! | `0x09`  | BLAKE2F         | Zcash bridge, privacy protocols              |
//!
//! ## Design
//!
//! `XnetPrecompiles<R>` implements `PrecompileSet` from `pallet_evm`. The EVM calls
//! `is_precompile` on every `CALL` targeting a low address, and if it returns `Some`
//! the call is dispatched here instead of to the bytecode executor.

#![cfg_attr(not(feature = "std"), )] 

use pallet_evm::{
    IsPrecompileResult, Precompile, PrecompileHandle, PrecompileResult, PrecompileSet,
};
use sp_core::H160;
use sp_std::marker::PhantomData;

// Standard Ethereum precompile implementations from Frontier.
use pallet_evm_precompile_blake2::Blake2F;
use pallet_evm_precompile_bn128::{Bn128Add, Bn128Mul, Bn128Pairing};
use pallet_evm_precompile_modexp::Modexp;
use pallet_evm_precompile_simple::{ECRecover, Identity, Ripemd160, Sha256};

/// The complete XNET EVM precompile set.
///
/// Generic over `R: pallet_evm::Config` so it can access runtime storage when
/// future custom precompiles (e.g. staking queries, ERC-20 wraps for native
/// XNET tokens) are added.
#[derive(Debug, Clone)]
pub struct XnetPrecompiles<R>(PhantomData<R>);

impl<R> Default for XnetPrecompiles<R> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<R> XnetPrecompiles<R> {
    /// Builds a new `XnetPrecompiles` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the list of precompile addresses that this set handles.
    ///
    /// Useful for chain-spec tools and integration tests that need to know which
    /// addresses are "special" without actually calling them.
    pub fn used_addresses() -> [H160; 9] {
        [
            hash(1),
            hash(2),
            hash(3),
            hash(4),
            hash(5),
            hash(6),
            hash(7),
            hash(8),
            hash(9),
        ]
    }
}

/// Constructs a 20-byte Ethereum address from a small integer.
/// Addresses `0x01`–`0x09` are the standard Ethereum precompile addresses.
fn hash(a: u64) -> H160 {
    H160::from_low_u64_be(a)
}

impl<R> PrecompileSet for XnetPrecompiles<R>
where
    R: pallet_evm::Config,
{
    /// Dispatches a call to the appropriate precompile, or returns `None` if
    /// the target address is not a precompile in this set.
    fn execute(&self, handle: &mut impl PrecompileHandle) -> Option<PrecompileResult> {
        match handle.code_address() {
            // 0x01 — ECRecover: recovers the signer address from an Ethereum signature.
            //        Required by MetaMask, wallets, and any contract doing sig verification.
            a if a == hash(1) => Some(ECRecover::execute(handle)),

            // 0x02 — SHA-256: returns the SHA-256 hash of arbitrary data.
            //        Used by Bitcoin bridge contracts and general hashing.
            a if a == hash(2) => Some(Sha256::execute(handle)),

            // 0x03 — RIPEMD-160: returns the RIPEMD-160 hash of arbitrary data.
            //        Used in Bitcoin address derivation.
            a if a == hash(3) => Some(Ripemd160::execute(handle)),

            // 0x04 — Identity: returns the input unchanged.
            //        Used internally by the Solidity ABI encoder.
            a if a == hash(4) => Some(Identity::execute(handle)),

            // 0x05 — ModExp: computes base^exp mod modulus for arbitrary precision integers.
            //        Required by RSA, some ZK circuits, and the EIP-198 standard.
            a if a == hash(5) => Some(Modexp::execute(handle)),

            // 0x06 — BN128Add: adds two points on the BN254 elliptic curve.
            //        Required for Groth16 ZK-SNARK verification (used by Tornado Cash,
            //        ZK-rollups, and many privacy protocols).
            a if a == hash(6) => Some(Bn128Add::execute(handle)),

            // 0x07 — BN128Mul: scalar multiplication on BN254.
            //        Required for ZK-SNARK verification.
            a if a == hash(7) => Some(Bn128Mul::execute(handle)),

            // 0x08 — BN128Pairing: bilinear pairing on BN254.
            //        The most expensive precompile — required for full Groth16 verification.
            //        Also used by Uniswap v3's internal math.
            a if a == hash(8) => Some(Bn128Pairing::execute(handle)),

            // 0x09 — BLAKE2F: the BLAKE2b F compression function.
            //        Required for Zcash bridge contracts and privacy-preserving protocols.
            a if a == hash(9) => Some(Blake2F::execute(handle)),

            // Any other address — not a precompile in this set.
            _ => None,
        }
    }

    /// Returns `Some(IsPrecompileResult)` if `address` is a precompile, `None` otherwise.
    ///
    /// The EVM calls this before every `CALL` to decide whether to invoke the bytecode
    /// executor or the precompile dispatcher.
    fn is_precompile(&self, address: H160, _gas: u64) -> IsPrecompileResult {
        if Self::used_addresses().contains(&address) {
            IsPrecompileResult::Answer {
                is_precompile: true,
                extra_cost: 0,
            }
        } else {
            IsPrecompileResult::Answer {
                is_precompile: false,
                extra_cost: 0,
            }
        }
    }
}
