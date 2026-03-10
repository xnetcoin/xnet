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
//! Matematika `ark-groth16` kutubxonasi orqali bajariladi:
//! bu kutubxona `no_std` + WASM muhitida to'liq ishlaydi.
//! `sp_io` host funksiyalariga bog'liqlik yo'q.
//!
//! ## Proof formati
//!
//! Proof bytes (256 bayt, Circom/snarkjs uncompressed affine format):
//!   [  0.. 64] A  — G1 nuqta (x: 32, y: 32)
//!   [ 64..192] B  — G2 nuqta (x_c1: 32, x_c0: 32, y_c1: 32, y_c0: 32)
//!   [192..256] C  — G1 nuqta (x: 32, y: 32)
//!
//! ## VK storage formati
//!
//! VK bytes — `ark_groth16::VerifyingKey<Bn254>` ning compressed borsh/canonical
//! serializatsiyasi. `snarkjs`ning `zkey export verificationkey` buyrug'idan
//! keyin Python/TS skript bilan ark-serialize formatiga aylantiriladi.

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

	use ark_bn254::{Bn254, Fq, Fq2, Fr, G1Affine, G2Affine};
	use ark_ff::PrimeField;
	use ark_groth16::{Groth16, PreparedVerifyingKey, Proof, VerifyingKey};
	use ark_serialize::CanonicalDeserialize;

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
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

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
		/// ark-serialize formatidagi canonical VerifyingKey baytlari.
		/// snarkjs VK → ark-vk-convert.py orqali hosil qilinadi.
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
		ProofVerified { vk_id: VkId, who: T::AccountId, block: BlockNumberFor<T> },

		/// Proof noto'g'ri — rad etildi.
		ProofRejected { vk_id: VkId, who: T::AccountId },

		/// Nullifier birinchi marta sarflandi.
		NullifierSpent { nullifier: Nullifier },

		/// Yangi VK ro'yxatdan o'tdi.
		VkRegistered { vk_id: VkId },

		/// VK o'chirildi.
		VkRemoved { vk_id: VkId },
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
		/// BN254 arifmetik hisob xatosi yoki deserializatsiya xatosi.
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
		/// # VK qanday tayyorlanadi
		///
		/// 1. Circuit Circom da yoziladi
		/// 2. `snarkjs groth16 setup circuit.r1cs pot.ptau circuit.zkey`
		/// 3. `snarkjs zkey export verificationkey circuit.zkey vk.json`
		/// 4. `python3 ark-vk-convert.py vk.json` → ark-serialize canonical baytlar
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

			// VK baytlarini parsedan o'tkazib tekshirish
			VerifyingKey::<Bn254>::deserialize_compressed(&vk_bytes[..])
				.map_err(|_| Error::<T>::InvalidVk)?;

			ensure!(ic_len >= 1, Error::<T>::InvalidVk);

			let bounded_vk: BoundedVec<u8, ConstU32<4096>> =
				vk_bytes.try_into().map_err(|_| Error::<T>::InvalidVk)?;

			let bounded_name: BoundedVec<u8, ConstU32<64>> = name.try_into().unwrap_or_default();

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

			// 6. Groth16 matematik tekshirish — ark-groth16 orqali
			let ok = Self::do_verify(&vk.vk_bytes, &proof, &public_inputs)
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
	// Groth16 matematik tekshirish — ark-groth16 + ark-bn254
	// ─────────────────────────────────────────────────────────────────────

	impl<T: Config> Pallet<T> {
		/// Groth16 tekshirish — `ark-groth16` orqali.
		///
		/// VK bytes — `VerifyingKey<Bn254>` ning compressed canonical formati.
		/// Proof bytes — 256 bayt: A(G1, 64) + B(G2, 128) + C(G1, 64).
		/// Public inputs — har biri 32 bayt big-endian `Fr` skalyari.
		pub fn do_verify(
			vk_bytes: &[u8],
			proof: &ProofBytes,
			inputs: &[[u8; 32]],
		) -> Result<bool, ()> {
			// VK deserializatsiya
			let vk = VerifyingKey::<Bn254>::deserialize_compressed(vk_bytes).map_err(|_| ())?;
			let pvk = PreparedVerifyingKey::from(vk);

			// Proof deserializatsiya (256 bayt uncompressed affine)
			let ark_proof = Self::decode_proof(proof)?;

			// Public inputs — Fr skalyarlarga aylantirish
			let pub_inputs = Self::decode_inputs(inputs)?;

			// Groth16 tekshirish
			Ok(Groth16::<Bn254>::verify_proof(&pvk, &ark_proof, &pub_inputs).unwrap_or(false))
		}

		/// Proof baytlarini `ark_groth16::Proof<Bn254>` ga aylantirish.
		///
		/// Kutilayotgan format (Circom/snarkjs uncompressed affine, big-endian):
		///   [  0.. 64] A : G1 — x(32) || y(32)
		///   [ 64..192] B : G2 — x_c1(32) || x_c0(32) || y_c1(32) || y_c0(32)
		///   [192..256] C : G1 — x(32) || y(32)
		fn decode_proof(proof: &ProofBytes) -> Result<Proof<Bn254>, ()> {
			// big-endian bytes → Fq (base field element) using PrimeField
			let read_fq = |b: &[u8]| -> Fq { Fq::from_be_bytes_mod_order(b) };

			// A — G1
			let ax = read_fq(&proof[0..32]);
			let ay = read_fq(&proof[32..64]);
			let a = G1Affine::new_unchecked(ax, ay);

			// B — G2 (x_c1 || x_c0 || y_c1 || y_c0)
			let bx_c1 = read_fq(&proof[64..96]);
			let bx_c0 = read_fq(&proof[96..128]);
			let by_c1 = read_fq(&proof[128..160]);
			let by_c0 = read_fq(&proof[160..192]);
			let bx = Fq2::new(bx_c0, bx_c1);
			let by = Fq2::new(by_c0, by_c1);
			let b = G2Affine::new_unchecked(bx, by);

			// C — G1
			let cx = read_fq(&proof[192..224]);
			let cy = read_fq(&proof[224..256]);
			let c = G1Affine::new_unchecked(cx, cy);

			Ok(Proof { a, b, c })
		}

		/// Public inputs baytlarini `Vec<Fr>` ga aylantirish.
		///
		/// Har bir input — 32 bayt big-endian Fr skalyari.
		fn decode_inputs(inputs: &[[u8; 32]]) -> Result<Vec<Fr>, ()> {
			Ok(inputs
				.iter()
				.map(|b| Fr::from_be_bytes_mod_order(b))
				.collect())
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
