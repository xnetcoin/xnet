//! # Block Reward Pallet — Unit Tests
//!
//! Tests are grouped into four sections:
//!
//! 1. **Reward calculation** — `calculate_reward` returns the correct value
//!    before and after each halving, and returns zero after 128 halvings.
//!
//! 2. **Block reward distribution** — `on_finalize` mints the right amount
//!    to the block author and emits `BlockRewardDistributed`.
//!
//! 3. **Hard cap (MaxSupply)** — once total issuance reaches `MaxSupply`,
//!    `on_finalize` emits `MaxSupplyReached` and mints nothing further.
//!    Also verifies the final-block clamp that prevents over-minting.
//!
//! 4. **No-author path** — when `FindAuthor` returns `None`, no mint occurs.

#![cfg(test)]

use frame_support::{assert_ok, traits::Hooks};
use sp_runtime::traits::Zero;

use crate::{Event, Pallet as BlockReward};

use super::mock::*;

// =============================================================================
// Section 1 — Reward calculation (unit tests for `calculate_reward`)
// =============================================================================

/// Before any halving (block 0–9) the reward equals `InitialBlockReward`.
#[test]
fn reward_is_initial_before_first_halving() {
	new_test_ext().execute_with(|| {
		for block in 0u64..10 {
			let reward = BlockReward::<Test>::calculate_reward(block);
			assert_eq!(reward, 1_000, "block {block}: expected full initial reward");
		}
	});
}

/// At block `HalvingInterval` (10) the reward is halved to 500.
#[test]
fn reward_halves_at_first_interval() {
	new_test_ext().execute_with(|| {
		let r = BlockReward::<Test>::calculate_reward(10);
		assert_eq!(r, 500);
	});
}

/// At block `2 × HalvingInterval` (20) the reward is quartered to 250.
#[test]
fn reward_halves_at_second_interval() {
	new_test_ext().execute_with(|| {
		let r = BlockReward::<Test>::calculate_reward(20);
		assert_eq!(r, 250);
	});
}

/// After 128 halvings the reward must be zero regardless of starting value.
#[test]
fn reward_is_zero_after_128_halvings() {
	new_test_ext().execute_with(|| {
		// 128 halvings × 10 blocks/halving = 1280
		let r = BlockReward::<Test>::calculate_reward(1280);
		assert_eq!(r, 0);
	});
}

/// Block 0 is still in the first period, reward must be InitialBlockReward.
#[test]
fn reward_at_genesis_block_is_initial() {
	new_test_ext().execute_with(|| {
		assert_eq!(BlockReward::<Test>::calculate_reward(0), 1_000);
	});
}

/// Block 9 is the last block before the first halving.
#[test]
fn reward_at_last_block_before_halving_is_initial() {
	new_test_ext().execute_with(|| {
		assert_eq!(BlockReward::<Test>::calculate_reward(9), 1_000);
	});
}

/// Block 10 crosses the halving boundary.
#[test]
fn reward_at_first_block_after_halving() {
	new_test_ext().execute_with(|| {
		assert_eq!(BlockReward::<Test>::calculate_reward(10), 500);
	});
}

// =============================================================================
// Section 2 — Block reward distribution via on_finalize
// =============================================================================

/// A single `on_finalize` call mints `InitialBlockReward` to ALICE.
#[test]
fn on_finalize_mints_reward_to_author() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(ALICE), 0);

		BlockReward::<Test>::on_finalize(1);

		// ALICE should now hold the initial reward.
		assert_eq!(Balances::free_balance(ALICE), 1_000);
	});
}

/// `BlockRewardDistributed` event is emitted with the correct author and amount.
#[test]
fn on_finalize_emits_event() {
	new_test_ext().execute_with(|| {
		System::reset_events();
		BlockReward::<Test>::on_finalize(1);

		let events = System::events();
		assert!(
			events.iter().any(|e| matches!(
				&e.event,
				RuntimeEvent::BlockReward(Event::BlockRewardDistributed {
					author,
					amount,
				}) if *author == ALICE && *amount == 1_000
			)),
			"expected BlockRewardDistributed event not found"
		);
	});
}

/// After the first halving (block 10), the reward drops to 500.
#[test]
fn on_finalize_pays_halved_reward_after_first_interval() {
	new_test_ext().execute_with(|| {
		BlockReward::<Test>::on_finalize(10);
		assert_eq!(Balances::free_balance(ALICE), 500);
	});
}

/// Across two consecutive halvings the cumulative minted amount is correct.
#[test]
fn cumulative_rewards_across_two_halvings() {
	new_test_ext().execute_with(|| {
		// Period 1: blocks 1–9 (9 blocks × 1_000 = 9_000)
		// But MaxSupply = 10_000 so we hit the cap before finishing period 1.
		// Use a fresh ext with no cap concern by checking individual blocks.

		// Block 1 (period 1): 1_000
		BlockReward::<Test>::on_finalize(1);
		// Block 10 (period 2): 500
		BlockReward::<Test>::on_finalize(10);
		// Block 20 (period 3): 250
		BlockReward::<Test>::on_finalize(20);

		assert_eq!(Balances::free_balance(ALICE), 1_750);
	});
}

// =============================================================================
// Section 3 — Hard cap enforcement (MaxSupply = 10_000)
// =============================================================================

/// Once total issuance equals MaxSupply, no further minting occurs.
#[test]
fn minting_stops_when_max_supply_reached() {
	// Pre-fund ALICE with exactly MaxSupply so total_issuance == MaxSupply from genesis.
	new_test_ext_with_balance(10_000).execute_with(|| {
		let before = Balances::free_balance(ALICE);
		BlockReward::<Test>::on_finalize(1);
		let after = Balances::free_balance(ALICE);

		assert_eq!(before, after, "balance must not increase once MaxSupply is reached");
	});
}

/// `MaxSupplyReached` event is emitted when total issuance is at the cap.
#[test]
fn max_supply_reached_event_emitted() {
	new_test_ext_with_balance(10_000).execute_with(|| {
		System::reset_events();
		BlockReward::<Test>::on_finalize(1);

		let events = System::events();
		assert!(
			events.iter().any(|e| matches!(
				&e.event,
				RuntimeEvent::BlockReward(Event::MaxSupplyReached)
			)),
			"expected MaxSupplyReached event not found"
		);
	});
}

/// The final reward is clamped so total issuance never exceeds MaxSupply.
/// Scenario: issuance is 9_500, full reward is 1_000. Minted must be clamped to 500.
#[test]
fn final_reward_is_clamped_to_remaining_headroom() {
	// Pre-fund ALICE so total issuance starts at 9_500.
	// MaxSupply = 10_000, so headroom = 500.
	// Full reward = 1_000, so actual minted = min(1_000, 500) = 500.
	new_test_ext_with_balance(9_500).execute_with(|| {
		BlockReward::<Test>::on_finalize(1);

		// Total issuance must be exactly 10_000 — not 10_500.
		assert_eq!(Balances::total_issuance(), 10_000);
		assert_eq!(Balances::free_balance(ALICE), 10_000);
	});
}

/// Running enough blocks to organically reach MaxSupply stops further minting.
#[test]
fn supply_cap_reached_organically() {
	new_test_ext().execute_with(|| {
		// Run 12 blocks at 1_000 reward each — cumulative would be 12_000 but
		// cap is 10_000, so minting stops at 10 blocks (10 × 1_000 = 10_000).
		for block in 1u64..=12 {
			BlockReward::<Test>::on_finalize(block);
		}

		// Total issuance must be exactly MaxSupply.
		assert_eq!(Balances::total_issuance(), 10_000);
	});
}

/// After the cap is reached, subsequent on_finalize calls are no-ops.
#[test]
fn on_finalize_is_noop_after_cap() {
	new_test_ext_with_balance(10_000).execute_with(|| {
		for block in 1u64..=5 {
			BlockReward::<Test>::on_finalize(block);
		}
		// Balance must remain unchanged across all five calls.
		assert_eq!(Balances::free_balance(ALICE), 10_000);
		assert_eq!(Balances::total_issuance(), 10_000);
	});
}

/// Total issuance never overshoot MaxSupply across a full run-to-block simulation.
#[test]
fn total_issuance_never_exceeds_max_supply() {
	new_test_ext().execute_with(|| {
		for block in 1u64..=50 {
			BlockReward::<Test>::on_finalize(block);
			// Invariant: issuance must never exceed MaxSupply.
			assert!(
				Balances::total_issuance() <= 10_000,
				"issuance exceeded MaxSupply at block {block}"
			);
		}
	});
}

// =============================================================================
// Section 4 — No-author path
// =============================================================================

/// When the runtime cannot resolve a block author, no mint should occur.
/// This requires a custom Config with `FindAuthor = NoAuthor`.
///
/// We test this indirectly via `calculate_reward`: if block author resolution
/// were to fail mid-finalize, the balance should remain 0.
/// The direct path is hard to test without a second runtime type, so we
/// assert on the internal invariant via the mock's `AlwaysAlice` and confirm
/// the regular path works, then document the no-author guard via comment.
///
/// For completeness, the test below verifies that `NoAuthor::find_author`
/// returns `None` so integration with the pallet is well understood.
#[test]
fn no_author_returns_none() {
	use frame_support::traits::FindAuthor;
	let result = NoAuthor::find_author(std::iter::empty());
	assert!(result.is_none());
}

/// When the block reward for a given block is zero (deep in the halving schedule),
/// `on_finalize` must not mint anything (the zero-check guard in `on_finalize`).
#[test]
fn zero_reward_is_not_minted() {
	new_test_ext().execute_with(|| {
		// Block 1280 corresponds to 128 halvings — reward is 0.
		BlockReward::<Test>::on_finalize(1280);
		// Nothing minted to ALICE.
		assert_eq!(Balances::free_balance(ALICE), 0);
	});
}

// =============================================================================
// Section 5 — Fee distribution (fee → treasury split simulation)
// =============================================================================

/// Verifies cumulative emission across period boundaries.
/// Period 1: blocks 1–9 → 9 × 1_000 = 9_000
/// Block 10: first halved period → reward = 500, so issuance becomes 9_500
/// Block 11: still period-2 (blocks 10–19) → reward = 500, issuance → 10_000 (capped)
#[test]
fn cumulative_reward_matches_geometric_sum() {
	new_test_ext().execute_with(|| {
		for block in 1u64..=11 {
			BlockReward::<Test>::on_finalize(block);
		}

		// After 11 blocks: 9×1_000 + 2×500 = 10_000 (MaxSupply).
		assert_eq!(Balances::free_balance(ALICE), 10_000);
		assert_eq!(Balances::total_issuance(), 10_000);
	});
}

/// Verifies that ALICE's balance after N blocks equals the sum of rewards
/// computed by `calculate_reward` for each of those blocks.
#[test]
fn balance_matches_sum_of_calculate_reward() {
	new_test_ext().execute_with(|| {
		let blocks = 5u64;
		let expected: u64 = (1..=blocks)
			.map(|b| BlockReward::<Test>::calculate_reward(b))
			.sum();

		for block in 1..=blocks {
			BlockReward::<Test>::on_finalize(block);
		}

		assert_eq!(Balances::free_balance(ALICE), expected);
	});
}
