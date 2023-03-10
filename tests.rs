use crate::{betting::Betting};

/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
    /// Imports all the definitions from the outer scope so we can use them here.
    use super::*;


    /// We test if the default constructor does its job.
    #[ink::test]
    fn constructor_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let betting = Betting::new();
        assert_eq!(betting.exists_match(accounts.alice), false);
    }

    #[ink::test]
    fn create_match_to_bet_works() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut betting = Betting::new();
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000000000000);

        betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10);
        assert_eq!(betting.exists_match(accounts.alice), true);
    }
}