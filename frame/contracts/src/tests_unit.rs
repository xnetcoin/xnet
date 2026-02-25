#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests {
    use crate::sample::Sample;

    /// Test: Balance should never exceed total supply
    #[ink::test]
    fn test_balance_never_exceeds_total_supply() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();

        // Mint to different account
        assert!(contract.mint(accounts.bob, 500));
        
        // Verify balances don't exceed total
        let alice_balance = contract.balance_of(accounts.alice);
        let bob_balance = contract.balance_of(accounts.bob);
        
        assert!(alice_balance <= contract.total_supply());
        assert!(bob_balance <= contract.total_supply());
        assert_eq!(alice_balance + bob_balance, contract.total_supply());
    }

    /// Test: Transfer preserves total supply
    #[ink::test]
    fn test_transfer_preserves_total_supply() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        let supply_before = contract.total_supply();
        
        // Transfer from alice to bob
        assert!(contract.transfer(accounts.bob, 100));
        
        let supply_after = contract.total_supply();
        assert_eq!(supply_before, supply_after);
    }

    /// Test: Transfer updates balances correctly
    #[ink::test]
    fn test_transfer_updates_balances() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        let alice_before = contract.balance_of(accounts.alice);
        let bob_before = contract.balance_of(accounts.bob);
        
        assert!(contract.transfer(accounts.bob, 250));
        
        let alice_after = contract.balance_of(accounts.alice);
        let bob_after = contract.balance_of(accounts.bob);
        
        assert_eq!(alice_after, alice_before - 250);
        assert_eq!(bob_after, bob_before + 250);
    }

    /// Test: Insufficient balance transfer fails
    #[ink::test]
    fn test_insufficient_balance_transfer_fails() {
        let mut contract = Sample::new(100);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        // Try to transfer more than balance
        assert!(!contract.transfer(accounts.bob, 1000));
    }

    /// Test: Mint increases total supply
    #[ink::test]
    fn test_mint_increases_supply() {
        let mut contract = Sample::new(100);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        let supply_before = contract.total_supply();
        assert!(contract.mint(accounts.bob, 500));
        let supply_after = contract.total_supply();
        
        assert_eq!(supply_after, supply_before + 500);
    }

    /// Test: Burn decreases total supply
    #[ink::test]
    fn test_burn_decreases_supply() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        let supply_before = contract.total_supply();
        assert!(contract.burn(accounts.alice, 200));
        let supply_after = contract.total_supply();
        
        assert_eq!(supply_after, supply_before - 200);
    }

    /// Test: Burn insufficient balance fails
    #[ink::test]
    fn test_burn_insufficient_balance_fails() {
        let mut contract = Sample::new(100);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        assert!(!contract.burn(accounts.alice, 500));
    }

    /// Test: Approve sets allowance
    #[ink::test]
    fn test_approve_sets_allowance() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        assert!(contract.approve(accounts.bob, 500));
        // Note: Allowance getter would need to be implemented in the contract
    }

    /// Test: Transfer from with sufficient allowance
    #[ink::test]
    fn test_transfer_from_with_allowance() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        assert!(contract.approve(accounts.bob, 500));
        
        // Switch caller to bob
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        
        assert!(contract.transfer_from(accounts.alice, accounts.charlie, 300));
    }

    /// Test: Transfer from without allowance fails
    #[ink::test]
    fn test_transfer_from_without_allowance_fails() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        // Don't approve
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        
        assert!(!contract.transfer_from(accounts.alice, accounts.charlie, 300));
    }

    /// Test: Transfer from exceeding allowance fails
    #[ink::test]
    fn test_transfer_from_exceeding_allowance_fails() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        assert!(contract.approve(accounts.bob, 200));
        
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        
        // Try to transfer more than approved
        assert!(!contract.transfer_from(accounts.alice, accounts.charlie, 500));
    }

    /// Test: Multiple transfers remain consistent
    #[ink::test]
    fn test_multiple_transfers_consistency() {
        let mut contract = Sample::new(1000);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        let total_before = contract.total_supply();
        
        assert!(contract.transfer(accounts.bob, 100));
        assert!(contract.transfer(accounts.charlie, 150));
        
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        assert!(contract.transfer(accounts.charlie, 50));
        
        let total_after = contract.total_supply();
        assert_eq!(total_before, total_after);
    }

    /// Test: Arithmetic overflow protection
    #[ink::test]
    fn test_arithmetic_safety() {
        let mut contract = Sample::new(u128::MAX - 100);
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        
        // This should not cause overflow
        // Contract implementation should handle this safely
        let _ = contract.mint(accounts.bob, 50);
    }
}
