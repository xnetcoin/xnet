//! # pallet-zk-verifier
//!
//! XNET Protocol uchun on-chain Groth16 ZK-SNARK tekshiruvchi.
//!
//! ## Qanday ishlaydi
//!
//! Groth16 tekshirish tenglamasi:
//!
//!   e(A, B) == e(α, β) · e(acc, γ) · e(C, δ)
//!
//! Bu tenglamani tekshirish uchun Substrate'ning o'z host funksiyalari
//! ishlatiladi — `sp_io::crypto::alt_bn128_*`. Bu Ethereum'ning
//! 0x06/0x07/0x08 precompile'lari bilan bir xil matematika.
//!
//! ## Tashqi kutubxona kerak emas
//!
//! `sp_io::crypto::alt_bn128_pairing` — to'g'ridan-to'g'ri WASM host function.
//! `sp_io::crypto::alt_bn128_mul`     — scalar ko'paytirish.
//! `sp_io::crypto::alt_bn128_add`     — nuqta qo'shish.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
pub use weights::WeightInfo;

pub mod weights;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use super::WeightInfo;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec;

    // ─────────────────────────────────────────────────────────────────────
    // Turlar
    // ─────────────────────────────────────────────────────────────────────

    /// Verification Key identifikatori — 32 bayt.
    /// Governance tomonidan tayinlanadi: keccak256("dark_pool_v1") kabi.
    pub type VkId = [u8; 32];

    /// Nullifier — double-spend himoyasi.
    /// Foydalanuvchi tomonidan hisoblanadi: Hash(secret || leaf_index).
    /// Bir marta sarflangandan keyin qayta ishlatib bo'lmaydi.
    pub type Nullifier = [u8; 32];

    /// Groth16 isboti baytlari:
    ///   A  : G1 nuqta — [  0.. 64]
    ///   B  : G2 nuqta — [ 64..192]
    ///   C  : G1 nuqta — [192..256]
    ///   Jami: 256 bayt
    pub type ProofBytes = [u8; 256];

    // ─────────────────────────────────────────────────────────────────────
    // Config
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>>
            + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Og'irlik ma'lumotlari — weights.rs dan keladi.
        type WeightInfo: WeightInfo;

        /// Bir blokda tekshirilishi mumkin bo'lgan maksimal proof soni.
        /// Spam va DoS hujumlaridan himoya.
        #[pallet::constant]
        type MaxProofsPerBlock: Get<u32>;

        /// Bir proof'dagi maksimal public inputs soni.
        #[pallet::constant]
        type MaxPublicInputs: Get<u32>;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Storage ma'lumot turi
    // ─────────────────────────────────────────────────────────────────────

    /// Storage'da saqlanadigan Verification Key.
    #[derive(Clone, Encode, Decode, TypeInfo, MaxEncodedLen, PartialEq, Debug)]
    #[scale_info(skip_type_params(T))]
    pub struct StoredVk {
        /// Seriallashtirilgan VK baytlari.
        /// Minimal hajm: alpha_g1(64) + beta_g2(128) + gamma_g2(128) + delta_g2(128) = 448 bayt
        /// Keyin IC elementlari: ic_len * 64 bayt
        #[codec(skip)]
        pub vk_bytes: BoundedVec<u8, ConstU32<4096>>,

        /// IC (Input-Commitments) elementlar soni.
        /// ic_len == public_inputs.len() + 1
        pub ic_len: u32,

        /// Circuit nomi — faqat ma'lumot uchun.
        #[codec(skip)]
        pub name: BoundedVec<u8, ConstU32<64>>,
    }

    // ─────────────────────────────────────────────────────────────────────
    // Pallet
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // ─────────────────────────────────────────────────────────────────────
    // Storage
    // ─────────────────────────────────────────────────────────────────────

    /// Ro'yxatdan o'tgan Verification Key'lar.
    #[pallet::storage]
    #[pallet::getter(fn vk)]
    pub type VerificationKeys<T: Config> =
        StorageMap<_, Blake2_128Concat, VkId, StoredVk, OptionQuery>;

    /// Sarflangan nullifier'lar.
    /// true = sarflangan, mavjud emas = sarflanmagan.
    #[pallet::storage]
    #[pallet::getter(fn nullifier_spent)]
    pub type SpentNullifiers<T: Config> =
        StorageMap<_, Blake2_128Concat, Nullifier, bool, ValueQuery>;

    /// Joriy blokdagi proof hisoblagichi — blok limiti uchun.
    #[pallet::storage]
    pub type ProofsThisBlock<T: Config> = StorageValue<_, u32, ValueQuery>;

    /// Jami muvaffaqiyatli tekshirilgan proof'lar.
    #[pallet::storage]
    #[pallet::getter(fn total_verified)]
    pub type TotalVerified<T: Config> = StorageValue<_, u64, ValueQuery>;

    // ─────────────────────────────────────────────────────────────────────
    // Hooks
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Har yangi blok boshida proof hisoblagichni nolga qaytarish.
        fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
            ProofsThisBlock::<T>::put(0u32);
            T::DbWeight::get().writes(1)
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Events
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Groth16 isboti muvaffaqiyatli tekshirildi.
        ProofVerified {
            vk_id: VkId,
            who: T::AccountId,
            block: BlockNumberFor<T>,
        },

        /// Proof noto'g'ri — rad etildi.
        ProofRejected {
            vk_id: VkId,
            who: T::AccountId,
        },

        /// Nullifier birinchi marta sarflandi.
        NullifierSpent {
            nullifier: Nullifier,
        },

        /// Yangi VK ro'yxatdan o'tdi.
        VkRegistered {
            vk_id: VkId,
        },

        /// VK o'chirildi.
        VkRemoved {
            vk_id: VkId,
        },
    }

    // ─────────────────────────────────────────────────────────────────────
    // Errors
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::error]
    pub enum Error<T> {
        /// Bu VK ID ro'yxatdan o'tmagan.
        VkNotFound,
        /// VK baytlari noto'g'ri — juda qisqa yoki format xato.
        InvalidVk,
        /// Proof 256 bayt bo'lishi kerak.
        InvalidProofLength,
        /// Public inputs soni VK'dagi IC soni bilan mos kelmaydi.
        InvalidPublicInputsCount,
        /// Public inputs soni ruxsat etilgan chegaradan oshdi.
        TooManyPublicInputs,
        /// Bu nullifier allaqachon sarflangan — double-spend urinishi.
        NullifierAlreadySpent,
        /// Joriy blokdagi proof limiti to'ldi.
        BlockLimitReached,
        /// Groth16 tenglamasi noto'g'ri — proof soxta.
        InvalidProof,
        /// BN254 arifmetik hisob xatosi.
        ArithmeticError,
    }

    // ─────────────────────────────────────────────────────────────────────
    // Calls
    // ─────────────────────────────────────────────────────────────────────

    #[pallet::call]
    impl<T: Config> Pallet<T> {

        /// Verification Key ro'yxatdan o'tkazish.
        ///
        /// Faqat Root (sudo yoki governance) chaqira oladi.
        ///
        /// # VK qanday olinadi
        ///
        /// 1. Circuit Circom da yoziladi
        /// 2. `snarkjs groth16 setup circuit.r1cs pot.ptau circuit.zkey`
        /// 3. `snarkjs zkey export verificationkey circuit.zkey vk.json`
        /// 4. vk.json → bytes ga aylantiriladi (vk_to_bytes.py skript)
        #[pallet::call_index(0)]
        #[pallet::weight(T::WeightInfo::register_vk())]
        pub fn register_vk(
            origin: OriginFor<T>,
            vk_id: VkId,
            vk_bytes: Vec<u8>,
            ic_len: u32,
            name: Vec<u8>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            // Minimal hajm: 448 bayt (alpha + beta + gamma + delta) + ic_len * 64
            let expected_min = 448usize + ic_len as usize * 64;
            ensure!(vk_bytes.len() >= expected_min, Error::<T>::InvalidVk);
            ensure!(ic_len >= 1, Error::<T>::InvalidVk);

            let bounded_vk: BoundedVec<u8, ConstU32<4096>> = vk_bytes
                .try_into()
                .map_err(|_| Error::<T>::InvalidVk)?;

            let bounded_name: BoundedVec<u8, ConstU32<64>> = name
                .try_into()
                .unwrap_or_default();

            VerificationKeys::<T>::insert(
                vk_id,
                StoredVk { vk_bytes: bounded_vk, ic_len, name: bounded_name },
            );

            Self::deposit_event(Event::VkRegistered { vk_id });
            Ok(())
        }

        /// VK o'chirish — faqat Root.
        #[pallet::call_index(1)]
        #[pallet::weight(T::WeightInfo::remove_vk())]
        pub fn remove_vk(origin: OriginFor<T>, vk_id: VkId) -> DispatchResult {
            ensure_root(origin)?;
            ensure!(VerificationKeys::<T>::contains_key(vk_id), Error::<T>::VkNotFound);
            VerificationKeys::<T>::remove(vk_id);
            Self::deposit_event(Event::VkRemoved { vk_id });
            Ok(())
        }

        /// Groth16 ZK isbotini on-chain tekshirish.
        ///
        /// # Argumentlar
        ///
        /// - `vk_id`        — qaysi circuit uchun tekshirish
        /// - `proof`        — 256 bayt: A(64) + B(128) + C(64)
        /// - `public_inputs`— ommaviy kirishlar (bytes32 massivi)
        /// - `nullifier`    — `Some(n)` = double-spend himoyasi bilan
        ///                    `None`    = nullifier kerak emas
        ///
        /// # Circom bilan ishlash
        ///
        /// ```bash
        /// # Proof yaratish
        /// snarkjs groth16 prove circuit.zkey input.json proof.json public.json
        ///
        /// # Bytes ga aylantirish
        /// node proof_to_bytes.js proof.json public.json
        /// ```
        #[pallet::call_index(2)]
        #[pallet::weight(T::WeightInfo::verify_proof(public_inputs.len() as u32))]
        pub fn verify_proof(
            origin: OriginFor<T>,
            vk_id: VkId,
            proof: ProofBytes,
            public_inputs: Vec<[u8; 32]>,
            nullifier: Option<Nullifier>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // 1. Blok limiti
            let count = ProofsThisBlock::<T>::get();
            ensure!(count < T::MaxProofsPerBlock::get(), Error::<T>::BlockLimitReached);

            // 2. Public inputs soni
            ensure!(
                public_inputs.len() <= T::MaxPublicInputs::get() as usize,
                Error::<T>::TooManyPublicInputs
            );

            // 3. VK olish
            let vk = VerificationKeys::<T>::get(vk_id).ok_or(Error::<T>::VkNotFound)?;

            // 4. IC soni mosligini tekshirish
            ensure!(
                public_inputs.len() + 1 == vk.ic_len as usize,
                Error::<T>::InvalidPublicInputsCount
            );

            // 5. Nullifier — HISOBLASHDAN OLDIN tekshirish (reentrancy himoyasi)
            if let Some(n) = nullifier {
                ensure!(!SpentNullifiers::<T>::get(n), Error::<T>::NullifierAlreadySpent);
                SpentNullifiers::<T>::insert(n, true);
                Self::deposit_event(Event::NullifierSpent { nullifier: n });
            }

            // 6. Groth16 matematik tekshirish
            let ok = Self::do_verify(&vk.vk_bytes, vk.ic_len, &proof, &public_inputs)
                .map_err(|_| Error::<T>::ArithmeticError)?;

            if !ok {
                Self::deposit_event(Event::ProofRejected { vk_id, who });
                return Err(Error::<T>::InvalidProof.into());
            }

            // 7. Statistika yangilash
            ProofsThisBlock::<T>::put(count + 1);
            TotalVerified::<T>::mutate(|n| *n += 1);

            Self::deposit_event(Event::ProofVerified {
                vk_id,
                who,
                block: frame_system::Pallet::<T>::block_number(),
            });

            Ok(())
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Groth16 matematik tekshirish
    // ─────────────────────────────────────────────────────────────────────

    impl<T: Config> Pallet<T> {

        /// Groth16 tekshirish.
        ///
        /// Tenglamani tekshiradi:
        ///   e(-A, B) · e(α, β) · e(acc, γ) · e(C, δ) == 1
        ///
        /// `sp_io::crypto::alt_bn128_pairing` — BN254 pairing host function.
        /// Bu Ethereum 0x08 precompile bilan bir xil.
        pub fn do_verify(
            vk: &[u8],
            ic_len: u32,
            proof: &ProofBytes,
            inputs: &[[u8; 32]],
        ) -> Result<bool, ()> {
            // VK tarkibi:
            //   [  0.. 64] alpha_g1 (G1)
            //   [ 64..192] beta_g2  (G2)
            //   [192..320] gamma_g2 (G2)
            //   [320..448] delta_g2 (G2)
            //   [448..   ] ic[0], ic[1], ..., ic[n]  (har biri 64 bayt)
            let alpha_g1 = &vk[0..64];
            let beta_g2  = &vk[64..192];
            let gamma_g2 = &vk[192..320];
            let delta_g2 = &vk[320..448];

            // Proof tarkibi:
            //   [  0.. 64] A (G1)
            //   [ 64..192] B (G2)
            //   [192..256] C (G1)
            let proof_a = &proof[0..64];
            let proof_b = &proof[64..192];
            let proof_c = &proof[192..256];

            // acc = ic[0] + Σ(inputs[i] * ic[i+1])
            let acc = Self::compute_linear_combination(vk, ic_len, inputs)?;

            // -A = A nuqtasini negatsiya qilish (y → p - y)
            let neg_a = Self::negate_g1(proof_a);

            // Pairing kirish: 4 juft × 192 bayt = 768 bayt
            // Har juft: G1(64 bayt) || G2(128 bayt)
            let mut input_buf = Vec::with_capacity(768);
            input_buf.extend_from_slice(&neg_a); // (-A, B)
            input_buf.extend_from_slice(proof_b);
            input_buf.extend_from_slice(alpha_g1); // (α, β)
            input_buf.extend_from_slice(beta_g2);
            input_buf.extend_from_slice(&acc); // (acc, γ)
            input_buf.extend_from_slice(gamma_g2);
            input_buf.extend_from_slice(proof_c); // (C, δ)
            input_buf.extend_from_slice(delta_g2);

            // BN254 pairing tekshirish — Substrate host function
            sp_io::crypto::alt_bn128_pairing(&input_buf).map_err(|_| ())
        }

        /// acc = ic[0] + Σ inputs[i] * ic[i+1]
        ///
        /// `alt_bn128_mul`  — scalar * G1 nuqta
        /// `alt_bn128_add`  — G1 + G1
        fn compute_linear_combination(
            vk: &[u8],
            ic_len: u32,
            inputs: &[[u8; 32]],
        ) -> Result<Vec<u8>, ()> {
            let ic_base = 448usize;

            // acc = ic[0]
            let mut acc = vk[ic_base..ic_base + 64].to_vec();

            for (i, scalar) in inputs.iter().enumerate() {
                let ic_i = &vk[ic_base + (i + 1) * 64..ic_base + (i + 2) * 64];

                // scalar * ic[i+1]
                // alt_bn128_mul kirish: G1(64) || scalar(32) = 96 bayt
                let mut mul_in = Vec::with_capacity(96);
                mul_in.extend_from_slice(ic_i);
                mul_in.extend_from_slice(scalar);
                let scaled = sp_io::crypto::alt_bn128_mul(&mul_in).map_err(|_| ())?;

                // acc = acc + scaled
                // alt_bn128_add kirish: G1(64) || G1(64) = 128 bayt
                let mut add_in = Vec::with_capacity(128);
                add_in.extend_from_slice(&acc);
                add_in.extend_from_slice(&scaled);
                acc = sp_io::crypto::alt_bn128_add(&add_in).map_err(|_| ())?;
            }

            let _ = ic_len;
            Ok(acc)
        }

        /// G1 nuqtasini negatsiya: (x, y) → (x, p - y)
        ///
        /// BN254 asosiy maydon moduli:
        /// p = 0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd47
        fn negate_g1(g1: &[u8]) -> Vec<u8> {
            // BN254 p
            const P: [u8; 32] = [
                0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29,
                0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
                0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d,
                0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c, 0xfd, 0x47,
            ];

            let x = &g1[0..32];
            let y = &g1[32..64];

            // p - y (big-endian)
            let mut neg_y = [0u8; 32];
            let mut borrow: u16 = 0;
            for i in (0..32).rev() {
                let diff = P[i] as u16 + 256 - y[i] as u16 - borrow;
                neg_y[i] = diff as u8;
                borrow = if diff < 256 { 1 } else { 0 };
            }

            let mut out = Vec::with_capacity(64);
            out.extend_from_slice(x);
            out.extend_from_slice(&neg_y);
            out
        }

        /// Tashqi kod uchun — nullifier sarflanganmi?
        pub fn is_nullifier_spent(n: Nullifier) -> bool {
            SpentNullifiers::<T>::get(n)
        }

        /// Tashqi kod uchun — VK mavjudmi?
        pub fn vk_exists(id: VkId) -> bool {
            VerificationKeys::<T>::contains_key(id)
        }
    }
}