/// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
/// module and test functions are marked with a `#[test]` attribute.
/// The below code is technically just normal Rust code.
#[cfg(test)]
mod tests {
    use crate::{betting::{Betting, Error, MatchResult, Bet}};
    use ink::primitives::AccountId;

    fn set_accounts() -> ink::env::test::DefaultAccounts<ink::env::DefaultEnvironment> {
        let accounts = ink::env::test::default_accounts::<ink::env::DefaultEnvironment>();
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.alice, 100000000000000,
        );
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.bob, 100000000000000,
        );
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.charlie, 100000000000000,
        );
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.django, 100000000000000,
        );
        ink::env::test::set_account_balance::<ink::env::DefaultEnvironment>(
            accounts.eve, 100000000000000,
        );
        accounts
    }

    fn create_contract(who: AccountId) -> Betting {
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(who);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(1000000000000);
        let betting = Betting::new();
        betting
    }


    fn create_match(betting: &mut Betting, who: AccountId, t1: &str, t2: &str, start: u32, length: u32, deposit: u128) -> AccountId {
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(who);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(deposit);
        // ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(deposit);
        // Dispatch a signed extrinsic.
        assert_eq!(betting.create_match_to_bet(t1.as_bytes().to_vec(), t2.as_bytes().to_vec(), start, length), Ok(()));
        who
    }

    /// We test if the default constructor does its job.
    #[ink::test]
    fn constructor_works() {
        let accounts = set_accounts();
        let betting = create_contract(accounts.alice);
        assert_eq!(betting.exists_match(accounts.alice), false);
    }

    #[ink::test]
    fn create_match_to_bet_works() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        assert_eq!(betting.exists_match(accounts.alice), false);

        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 10, 10, 1000000000000);

        assert_eq!(betting.exists_match(match_id), true);

        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert_eq!(1, emitted_events.len());

    }

    #[ink::test]
    fn not_enough_deposit_when_create_match_to_bet() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(1);

        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 10, 10),
            Err(Error::NotEnoughDeposit)
        );
        assert_eq!(betting.exists_match(accounts.alice), false);
    }

    #[ink::test]
    fn match_exist_when_create_match_to_bet() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        create_match(&mut betting, accounts.alice, "team1", "team2", 10, 10, 1000000000000);

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

        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);
        
        assert_eq!(betting.exists_match(accounts.alice), false);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(1000000000000);

        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 1, 1),
            Err(Error::TimeMatchOver)
        );
        assert_eq!(betting.exists_match(accounts.alice), false);
    }
    #[ink::test]
    fn error_creating_two_equal_matches() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);
        
        assert_eq!(betting.exists_match(accounts.alice), false);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(1000000000000);
        assert_eq!(betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 1, 1), Ok(()));

        //Bob to create same match as alice just created
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(1000000000000);

        assert_eq!(
            betting.create_match_to_bet("team1".as_bytes().to_vec(), "team2".as_bytes().to_vec(), 1, 1),
            Err(Error::MatchAlreadyExists)
        );
    }

    #[ink::test]
    fn bet_works() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 10, 10, 1000000000000);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));

        let bet = Bet {
            bettor: accounts.bob,
            amount: 10000000000,
            result: MatchResult::Team1Victory,
        };
        assert_eq!(betting.get_match(match_id).unwrap().bets.contains(&bet), true);

        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert_eq!(2, emitted_events.len());

    }

    #[ink::test]
    fn bet_error_match_not_exist() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000);
        assert_eq!(betting.bet(accounts.alice, MatchResult::Team1Victory),  Err(Error::MatchDoesNotExist));
    }

    #[ink::test]
    fn bet_error_match_has_start() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        create_match(&mut betting, accounts.alice, "team1", "team2", 1, 10, 1000000000000);
        // Advance 2 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000);
        assert_eq!(betting.bet(accounts.alice, MatchResult::Team1Victory),  Err(Error::MatchHasStarted));
    }

    #[ink::test]
    fn bet_error_duplicate_bet() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 10, 10, 1000000000000);

        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));

        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory),  Err(Error::AlreadyBet));

    }

    #[ink::test]
    fn set_result_works() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 1, 1, 1000000000000);

        assert_eq!(betting.exists_match(match_id), true);

        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        assert_eq!(betting.set_result(match_id, MatchResult::Team1Victory), Ok(()));

        let emitted_events = ink::env::test::recorded_events().collect::<Vec<_>>();
        assert_eq!(2, emitted_events.len());

    }
    #[ink::test]
    fn set_result_bad_origin() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 1, 1, 1000000000000);

        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        //set Bob as the caller
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        assert_eq!(betting.set_result(match_id, MatchResult::Team1Victory), Err(Error::BadOrigin));
    }
    #[ink::test]
    fn set_result_match_not_exist() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        assert_eq!(betting.set_result(accounts.alice, MatchResult::Team1Victory), Err(Error::MatchDoesNotExist));
    }
    #[ink::test]
    fn set_result_match_not_finished() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);
        let match_id = create_match(&mut betting, accounts.alice, "team1", "team2", 10, 10, 1000000000000);

        assert_eq!(betting.set_result(match_id, MatchResult::Team1Victory), Err(Error::TimeMatchNotOver));
    }

    #[ink::test]
    fn distribute_winnings_works() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        //Django creates the match
        let match_id = create_match(&mut betting, accounts.django, "team1", "team2", 1, 1, 1000000000000);

        assert_eq!(betting.exists_match(match_id), true);
        // Bob bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));
        // Charlie bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team2Victory), Ok(()));
        // Eve bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(30000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));

        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        //Alice set the result
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        assert_eq!(betting.set_result(match_id, MatchResult::Team1Victory), Ok(()));
        //Django distributes the winnings
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        assert_eq!(betting.distribute_winnings(), Ok(()));
        //bob has 90 + 12.5 (winner)
        assert_eq!(ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.bob), Ok(102500000000000));
        //charlie has 90 (loser)
        assert_eq!(ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.charlie), Ok(90000000000000));
        //eve has 90 + 37.5 (winner)
        assert_eq!(ink::env::test::get_account_balance::<ink::env::DefaultEnvironment>(accounts.eve), Ok(107500000000000));

    }

    #[ink::test]
    fn distribute_winnings_match_not_exist() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        //Django creates the match
        let match_id = create_match(&mut betting, accounts.django, "team1", "team2", 1, 1, 1000000000000);

        assert_eq!(betting.exists_match(match_id), true);
        // Bob bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));
        // Charlie bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team2Victory), Ok(()));
        // Eve bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(30000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));

        // Advance 3 blocks
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        ink::env::test::advance_block::<ink::env::DefaultEnvironment>();
        //Alice set the result
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.alice);
        assert_eq!(betting.set_result(match_id, MatchResult::Team1Victory), Ok(()));

        //alice distribute winner doesn't exists
        assert_eq!(betting.distribute_winnings(), Err(Error::MatchDoesNotExist));

    }

    #[ink::test]
    fn distribute_winnings_match_not_result_yet() {
        let accounts = set_accounts();
        let mut betting = create_contract(accounts.alice);

        //Django creates the match
        let match_id = create_match(&mut betting, accounts.django, "team1", "team2", 1, 1, 1000000000000);

        assert_eq!(betting.exists_match(match_id), true);
        // Bob bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.bob);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));
        // Charlie bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.charlie);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(10000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team2Victory), Ok(()));
        // Eve bets
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.eve);
        ink::env::test::transfer_in::<ink::env::DefaultEnvironment>(30000000000000);
        assert_eq!(betting.bet(match_id, MatchResult::Team1Victory), Ok(()));

        //Django distributes the winnings
        ink::env::test::set_caller::<ink::env::DefaultEnvironment>(accounts.django);
        assert_eq!(betting.distribute_winnings(), Err(Error::MatchNotResult));

    }
}