#![cfg_attr(not(feature = "std"), no_std)]

use ink::prelude::*;

#[ink::contract]
pub mod sample {
    use ink::storage::Mapping;

    #[ink(storage)]
    pub struct Sample {
        total_supply: Balance,
        balances: Mapping<AccountId, Balance>,
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    impl Sample {
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self {
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &initial_supply);

            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: initial_supply,
            });

            Self {
                total_supply: initial_supply,
                balances,
                allowances: Mapping::default(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balances.get(owner).unwrap_or(0)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> bool {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            true
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> bool {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> bool {
            let caller = self.env().caller();
            let allowance = self.allowances.get((&from, &caller)).unwrap_or(0);

            if allowance < value {
                return false;
            }

            self.allowances
                .insert((&from, &caller), &(allowance - value));
            self.transfer_from_to(&from, &to, value)
        }

        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> bool {
            let from_balance = self.balance_of(*from);
            if from_balance < value {
                return false;
            }

            self.balances.insert(from, &(from_balance - value));

            let to_balance = self.balance_of(*to);
            self.balances.insert(to, &(to_balance + value));

            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });

            true
        }

        #[ink(message)]
        pub fn mint(&mut self, to: AccountId, value: Balance) -> bool {
            let to_balance = self.balance_of(to);
            self.balances.insert(&to, &(to_balance + value));
            self.total_supply += value;

            self.env().emit_event(Transfer {
                from: None,
                to: Some(to),
                value,
            });

            true
        }

        #[ink(message)]
        pub fn burn(&mut self, from: AccountId, value: Balance) -> bool {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return false;
            }

            self.balances.insert(&from, &(from_balance - value));
            self.total_supply -= value;

            self.env().emit_event(Transfer {
                from: Some(from),
                to: None,
                value,
            });

            true
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink::test]
        fn it_works() {
            let contract = Sample::new(100);
            assert_eq!(contract.total_supply(), 100);
        }

        #[ink::test]
        fn balance_works() {
            let contract = Sample::new(100);
            assert_eq!(contract.balance_of(Self::env().caller()), 100);
        }
    }
}
