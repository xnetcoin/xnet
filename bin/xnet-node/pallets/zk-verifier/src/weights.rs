//! # weights.rs
//!
//! ZKP operatsiyalari uchun og'irlik (Weight) ma'lumotlari.
//!
//! ## Nima uchun kerak
//!
//! Groth16 tekshirish — og'ir matematik operatsiya:
//! 4 ta BN254 pairing + n ta scalar ko'paytirish + n ta nuqta qo'shish.
//!
//! Agar og'irlik to'g'ri o'lchanmasa:
//! - Xaker arzon extrinsic bilan tarmoqni to'xtatib qo'ya oladi
//! - Bloklar kerakligidan ko'p vaqt oladi
//!
//! ## Qiymatlar
//!
//! Haqiqiy benchmark uchun: `cargo benchmark` ishlatiladi.
//! Hozir konservativ (xavfsiz) qiymatlar yozilgan.
//!
//! Benchmark ishlatish:
//! ```bash
//! cargo build --release --features runtime-benchmarks
//! ./target/release/xnet-node benchmark pallet \
//!     --pallet pallet_zk_verifier \
//!     --extrinsic "*" \
//!     --steps 50 --repeat 20 \
//!     --output pallets/zk-verifier/src/weights.rs
//! ```

use frame_support::weights::Weight;

/// Pallet og'irlik interfeysi.
pub trait WeightInfo {
    fn register_vk() -> Weight;
    fn remove_vk() -> Weight;
    fn verify_proof(n: u32) -> Weight; // n = public inputs soni
}

/// Substrat benchmark orqali o'lchangan haqiqiy og'irliklar.
/// Hozir konservativ qiymatlar — benchmark dan keyin yangilanadi.
pub struct SubstrateWeight<T>(core::marker::PhantomData<T>);

impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {

    /// VK ro'yxatdan o'tkazish.
    ///
    /// Operatsiyalar: 1 storage write, VK bytes decode.
    /// ref_time: ~5ms konservativ.
    fn register_vk() -> Weight {
        Weight::from_parts(5_000_000, 0)
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// VK o'chirish.
    ///
    /// Operatsiyalar: 1 storage check + 1 storage delete.
    fn remove_vk() -> Weight {
        Weight::from_parts(2_000_000, 0)
            .saturating_add(T::DbWeight::get().reads(1))
            .saturating_add(T::DbWeight::get().writes(1))
    }

    /// Groth16 proof tekshirish.
    ///
    /// ## Og'irlik tarkibi
    ///
    /// | Operatsiya            | Taxminiy vaqt |
    /// |-----------------------|---------------|
    /// | 4x BN254 pairing      | ~150ms        |
    /// | n x scalar mul        | ~5ms * n      |
    /// | n x point add         | ~1ms * n      |
    /// | Storage reads         | ~0.5ms        |
    ///
    /// n = public_inputs soni
    ///
    /// Konservativ: 200ms bazaviy + 6ms * n
    fn verify_proof(n: u32) -> Weight {
        let base = Weight::from_parts(200_000_000, 0);
        let per_input = Weight::from_parts(6_000_000, 0);

        base.saturating_add(per_input.saturating_mul(n as u64))
            .saturating_add(T::DbWeight::get().reads(2))  // VK + nullifier
            .saturating_add(T::DbWeight::get().writes(2)) // nullifier + counter
    }
}

/// Test uchun og'irliklar — hamma operatsiya 0 weight.
pub struct TestWeightInfo;

impl WeightInfo for TestWeightInfo {
    fn register_vk() -> Weight { Weight::zero() }
    fn remove_vk()   -> Weight { Weight::zero() }
    fn verify_proof(_n: u32) -> Weight { Weight::zero() }
}