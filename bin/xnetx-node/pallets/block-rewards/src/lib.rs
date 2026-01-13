//! # XnetXCoin Block Rewards Pallet
//!
//! This pallet implements the block reward mechanism with halving for XnetXCoin (XNX).
//!
//! ## Overview
//!
//! - Initial block reward: 1.565 XNX
//! - Halving interval: Every 4 years (10,512,000 blocks)
//! - Total mining supply: 47,000,000 XNX
//! - Block time: 12 seconds

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, Get},
	};
	use frame_system::pallet_prelude::*;
	use sp_runtime::traits::Zero;
    use frame_support::traits::FindAuthor; // find_author funksiyasi uchun
    use sp_runtime::traits::Saturating;   // saturating_add va saturating_sub uchun
	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The currency mechanism.
		type Currency: Currency<Self::AccountId>;

		/// Initial block reward (with decimals). 1.565 XNX = 1_565_000_000_000_000_000
		#[pallet::constant]
		type InitialBlockReward: Get<BalanceOf<Self>>;

		/// Halving interval in blocks. 4 years = 10_512_000 blocks
		#[pallet::constant]
		type HalvingInterval: Get<BlockNumberFor<Self>>;

		/// Maximum total emission for mining. 47,000,000 XNX
		#[pallet::constant]
		type MaxEmission: Get<BalanceOf<Self>>;

		/// Find the author of a block (from Authorship pallet).
		type FindAuthor: frame_support::traits::FindAuthor<Self::AccountId>;
	}

	pub type BalanceOf<T> =
		<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Total amount of XNX minted through block rewards
	#[pallet::storage]
	#[pallet::getter(fn total_minted)]
	pub type TotalMinted<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	/// Current halving era (0 = first 4 years, 1 = second 4 years, etc.)
	#[pallet::storage]
	#[pallet::getter(fn current_era)]
	pub type CurrentEra<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Block reward was issued to the block author
		BlockRewardIssued { 
			block_number: BlockNumberFor<T>,
			author: T::AccountId, 
			reward: BalanceOf<T>,
		},
		/// Halving occurred. [new_era, new_reward]
		HalvingOccurred { era: u32, new_reward: BalanceOf<T> },
		/// Max emission reached, no more rewards.
		MaxEmissionReached,
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Maximum emission has been reached.
		MaxEmissionReached,
		/// Arithmetic overflow.
		Overflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		/// Called at the end of each block to issue rewards to the block author
		fn on_finalize(block_number: BlockNumberFor<T>) {
			// Get the digest from the block and find the author
			let digest = frame_system::Pallet::<T>::digest();
			let pre_runtime_digests = digest.logs.iter().filter_map(|d| d.as_pre_runtime());
			
			if let Some(author) = T::FindAuthor::find_author(pre_runtime_digests) {
				// Issue reward to the block author
				let _ = Self::issue_block_reward_to_author(block_number, &author);
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Calculate the current block reward based on halving schedule
		pub fn calculate_current_reward(block_number: BlockNumberFor<T>) -> BalanceOf<T> {
			let halving_interval = T::HalvingInterval::get();
			let initial_reward = T::InitialBlockReward::get();

			// Calculate which era we're in (0, 1, 2, 3, ...)
			let era: u32 = (block_number / halving_interval)
				.try_into()
				.unwrap_or(0u32);

			// Reward = initial_reward / 2^era
			// After era 20+, reward is essentially 0
			if era >= 20 {
				return Zero::zero();
			}

			// Divide by 2^era
			let divisor: u128 = 1u128 << era;
			let initial_u128: u128 = initial_reward.try_into().unwrap_or(0);
			let reward_u128 = initial_u128 / divisor;

			reward_u128.try_into().unwrap_or(Zero::zero())
		}

		/// Issue block reward to the block author
		fn issue_block_reward_to_author(block_number: BlockNumberFor<T>, author: &T::AccountId) -> DispatchResult {
			let max_emission = T::MaxEmission::get();
			let total_minted = Self::total_minted();

			// Check if we've reached max emission
			if total_minted >= max_emission {
				Self::deposit_event(Event::MaxEmissionReached);
				return Ok(());
			}

			let reward = Self::calculate_current_reward(block_number);

			// Don't issue if reward is zero
			if reward.is_zero() {
				return Ok(());
			}

			// Calculate remaining emission capacity
			let remaining = max_emission.saturating_sub(total_minted);
			let actual_reward = if reward > remaining { remaining } else { reward };

			// Mint the reward directly to the block author
			let _ = T::Currency::deposit_creating(author, actual_reward);

			// Update total minted
			TotalMinted::<T>::put(total_minted.saturating_add(actual_reward));

			// Emit block reward event
			Self::deposit_event(Event::BlockRewardIssued {
				block_number,
				author: author.clone(),
				reward: actual_reward,
			});

			// Check for era change and emit event
			let halving_interval = T::HalvingInterval::get();
			let new_era: u32 = (block_number / halving_interval)
				.try_into()
				.unwrap_or(0u32);
			let current_era = Self::current_era();

			if new_era > current_era {
				CurrentEra::<T>::put(new_era);
				Self::deposit_event(Event::HalvingOccurred {
					era: new_era,
					new_reward: Self::calculate_current_reward(block_number),
				});
			}

			Ok(())
		}

		/// Get total XNX minted so far
		pub fn get_total_minted() -> BalanceOf<T> {
			Self::total_minted()
		}

		/// Get remaining emission
		pub fn get_remaining_emission() -> BalanceOf<T> {
			T::MaxEmission::get().saturating_sub(Self::total_minted())
		}

		/// Get current era number
		pub fn get_current_era() -> u32 {
			Self::current_era()
		}
	}
}
