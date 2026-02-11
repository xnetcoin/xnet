#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use frame_support::traits::{Currency, FindAuthor};
    use sp_runtime::traits::{SaturatedConversion, Zero};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    // BalanceOf 
    pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type Currency: Currency<Self::AccountId>;

        type FindAuthor: FindAuthor<Self::AccountId>;

        #[pallet::constant]
        type InitialBlockReward: Get<BalanceOf<Self>>;

        #[pallet::constant]
        type HalvingInterval: Get<BlockNumberFor<Self>>;
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        BlockRewardDistributed { author: T::AccountId, amount: BalanceOf<T> },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(block_number: BlockNumberFor<T>) {
            let digest = frame_system::Pallet::<T>::digest();
            let pre_runtime_digests = digest.logs().iter().filter_map(|d| d.as_pre_runtime());

            if let Some(author) = T::FindAuthor::find_author(pre_runtime_digests) {
                let reward = Self::calculate_reward(block_number);

                let _ = T::Currency::deposit_creating(&author, reward);

                Self::deposit_event(Event::BlockRewardDistributed { author, amount: reward });
            }
        }
    }

    impl<T: Config> Pallet<T> {
        fn calculate_reward(block_number: BlockNumberFor<T>) -> BalanceOf<T> {
            let initial_reward = T::InitialBlockReward::get();
            let halving_interval = T::HalvingInterval::get();

            if halving_interval.is_zero() {
                return initial_reward;
            }

            let halvings = block_number / halving_interval;

            let shifts = match TryInto::<u32>::try_into(halvings) {
                Ok(s) => s,
                Err(_) => return BalanceOf::<T>::from(0u32), 
            };

            if shifts >= 128 {
                return BalanceOf::<T>::from(0u32);
            }

            let amount_u128: u128 = initial_reward.saturated_into::<u128>();
            let final_amount = amount_u128 >> shifts;

            final_amount.saturated_into()
        }
    }
}