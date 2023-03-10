use crate::{betting::{Betting, Error}};

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

        assert_eq!(betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10), Ok(()));
        assert_eq!(betting.exists_match(accounts.alice), true);

        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert_eq!(1, emitted_events.len());

    }

    #[ink::test]
    fn not_enough_deposit_when_create_match_to_bet() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut betting = Betting::new();
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1);

        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10),
            Err(Error::NotEnoughDeposit)
        );
        assert_eq!(betting.exists_match(accounts.alice), false);
    }

    #[ink::test]
    fn match_exist_when_create_match_to_bet() {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut betting = Betting::new();
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000000000000);

        assert_eq!(betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10), Ok(()));
        assert_eq!(betting.exists_match(accounts.alice), true);

        //Try to added it again
        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10),
            Err(Error::OriginHasAlreadyOpenMatch)
        );
    }

    #[ink::test]
    fn error_creating_a_match_with_an_open_match() {
        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();

        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        let mut betting = Betting::new();
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::set_value_transferred::<ink::env::DefaultEnvironment>(1000000000000);

        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 1, 1),
            Err(Error::TimeMatchOver)
        );
        assert_eq!(betting.exists_match(accounts.alice), false);
    }
}