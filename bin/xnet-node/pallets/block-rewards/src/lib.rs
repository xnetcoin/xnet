//! # Block Reward Pallet
//!
//! Mints a per-block reward for the block author and enforces XNET's fixed
//! maximum supply of **53,000,000 XNET**.
//!
//! ## Emission Schedule
//!
//! The reward follows a Bitcoin-style halving schedule:
//!
//! | Period | Blocks | Reward/block | Total minted |
//! |--------|--------|-------------|--------------|
//! | 1st    | 21,038,400 | 1.117 XNET | ~23.5M XNET |
//! | 2nd    | 21,038,400 | 0.5585 XNET | ~11.75M XNET |
//! | 3rd    | 21,038,400 | 0.2793 XNET | ~5.87M XNET |
//! | ...    | ...    | ...         | ... |
//! | All periods combined (geometric sum) | ~47M XNET |
//!
//! Combined with the genesis premine of **6,000,000 XNET**, the total hard cap
//! is **53,000,000 XNET**. Once total issuance reaches that ceiling the reward
//! is set to zero and no further minting occurs.
//!
//! ## Safety
//!
//! Every `on_finalize` call checks `total_issuance ≥ MaxSupply` before
//! calling `deposit_creating`, so the cap is enforced at the pallet level
//! independently of the halving calculation.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency, FindAuthor};
    use sp_runtime::traits::{SaturatedConversion, Zero};
    use sp_runtime::Saturating;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Convenience alias — the balance type of the runtime's currency.
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// The currency used for minting block rewards (the native XNET token).
        type Currency: Currency<Self::AccountId>;

        /// Resolves the block author's `AccountId` from the block digest.
        type FindAuthor: FindAuthor<Self::AccountId>;

        /// Block reward issued at genesis, before any halving occurs.
        #[pallet::constant]
        type InitialBlockReward: Get<BalanceOf<Self>>;

        /// Number of blocks between consecutive halvings.
        /// At 6-second blocks this is approximately 4 years.
        #[pallet::constant]
        type HalvingInterval: Get<BlockNumberFor<Self>>;

        /// Absolute maximum token supply. Minting stops permanently once
        /// total issuance reaches this value.
        /// For XNET this is set to 53,000,000 × 10^18 planck.
        #[pallet::constant]
        type MaxSupply: Get<BalanceOf<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A block reward was minted and sent to the block author.
        BlockRewardDistributed { author: T::AccountId, amount: BalanceOf<T> },

        /// Total issuance has reached `MaxSupply`. No further rewards will be minted.
        MaxSupplyReached,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        /// Called at the end of every block.
        ///
        /// 1. Reads total issuance and compares it to `MaxSupply`.
        /// 2. Clamps the halving-adjusted reward so it never pushes issuance above the cap.
        /// 3. Deposits the clamped reward to the block author.
        fn on_finalize(block_number: BlockNumberFor<T>) {
            // --- Supply cap guard ---
            let total_issuance = T::Currency::total_issuance();
            let max_supply = T::MaxSupply::get();

            if total_issuance >= max_supply {
                // Cap already reached — emit event once and return early.
                Self::deposit_event(Event::MaxSupplyReached);
                return;
            }

            // --- Author resolution ---
            let digest = frame_system::Pallet::<T>::digest();
            let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());

            if let Some(author) = T::FindAuthor::find_author(pre_runtime_digests) {
                // --- Halving-adjusted reward ---
                let reward = Self::calculate_reward(block_number);

                // --- Clamp to remaining supply headroom ---
                // This prevents the very last reward from minting more than allowed.
                let remaining = max_supply.saturating_sub(total_issuance);
                let actual_reward = reward.min(remaining);

                if actual_reward.is_zero() {
                    return;
                }

                // Mint (deposit_creating increases total issuance).
                let _ = T::Currency::deposit_creating(&author, actual_reward);

                Self::deposit_event(Event::BlockRewardDistributed {
                    author,
                    amount: actual_reward,
                });
            }
        }
    }

    impl<T: Config> Pallet<T> {
        /// Returns the halving-adjusted block reward for the given block number.
        ///
        /// Each halving divides the previous reward by two (right-bit-shift).
        /// After 128 halvings the result is guaranteed to be zero.
        fn calculate_reward(block_number: BlockNumberFor<T>) -> BalanceOf<T> {
            let initial_reward = T::InitialBlockReward::get();
            let halving_interval = T::HalvingInterval::get();

            // Guard: misconfigured zero interval → use initial reward unchanged.
            if halving_interval.is_zero() {
                return initial_reward;
            }

            // How many complete halving periods have elapsed?
            let halvings = block_number / halving_interval;

            let shifts = match TryInto::<u32>::try_into(halvings) {
                Ok(s) => s,
                // halvings overflowed u32 → reward is effectively zero.
                Err(_) => return BalanceOf::<T>::from(0u32),
            };

            // After 128 halvings the u128 value is zero regardless of starting value.
            if shifts >= 128 {
                return BalanceOf::<T>::from(0u32);
            }

            let amount_u128: u128 = initial_reward.saturated_into::<u128>();
            let final_amount = amount_u128 >> shifts;

            final_amount.saturated_into()
        }
    }
}