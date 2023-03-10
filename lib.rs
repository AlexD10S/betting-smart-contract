#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

#[ink::contract]
mod betting {
    use ink::storage::Mapping;

    // Use BoundedVec?
    pub type TeamName = Vec<u8>;

    const MIN_DEPOSIT: Balance = 1_000_000_000_000;

    #[derive(scale::Decode, scale::Encode, PartialEq, Clone)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum MatchResult {
        Team1Victory,
        Team2Victory,
        Draw,
    }
    #[derive(scale::Decode, scale::Encode, PartialEq)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Bet {
        /// Account of the better.
        bettor: AccountId,
        /// Bet amount.
        amount: Balance,
        /// Result predicted.
        result: MatchResult,
    }
    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub struct Match {
        /// Starting block of the match.
        start: BlockNumber,
        /// Length of the match (start + length = end).
        length: BlockNumber,
        /// Team1 name.
        team1: TeamName,
        /// Team2 name.
        team2: TeamName,
        /// Result.
        result: Option<MatchResult>,
        /// List of bets.
        bets: Vec<Bet>,
        /// The amount held in reserve of the `depositor`,
        /// To be returned once this recovery process is closed.
        deposit: Balance,
    }

    #[ink(storage)]
    pub struct Betting {
        /// Mapping of open matches.
        matches: Mapping<AccountId, Match>,
        // Mapping of all match hashes. (hash -> owner)
        //matches_hashes: Mapping<Hash, AccountId>
    }

    /// A new match has been created. [who, team1, team2, start, length]
    #[ink(event)]
    pub struct MatchCreated {
        #[ink(topic)]
        who: AccountId,
        team1: TeamName,
        team2: TeamName,
        start: BlockNumber,
        length: BlockNumber
    }
    /// A new bet has been created. [matchId, who, amount, result]
    #[ink(event)]
    pub struct BetPlaced {
        #[ink(topic)]
        match_id: AccountId,
        #[ink(topic)]
        who: AccountId,
        amount: Balance,
        result: MatchResult,
    }

    /// The Betting error types.
    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// The match to be created already exist.
        MatchAlreadyExists,
        /// Each account can only have one match open.
        OriginHasAlreadyOpenMatch,
        /// The time of the match is over.
        TimeMatchOver,
        /// Not enough deposit to create the Match.
        NotEnoughDeposit,
        /// The match where the bet is placed does not exist
        MatchDoesNotExist,
        /// No allowing betting if the match has started
        MatchHasStarted,
        /// You already place the same bet in that match
        AlreadyBet,
    }

    impl Betting {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                matches: Default::default(),
                //matches_hashes: Default::default(),
            }
        }

        // payable accepts a payment (deposit).
        #[ink(message, payable)]
        pub fn create_match_to_bet(
            &mut self, 
            team1: Vec<u8>,
            team2: Vec<u8>,
            start: BlockNumber,
            length: BlockNumber
        ) -> Result<(), Error> {
            let caller = Self::env().caller();
            // Check account has no open match
            if self.exists_match(caller) {
                return Err(Error::OriginHasAlreadyOpenMatch)
            }
            // Check if start and length are valid
            let current_block_number = self.env().block_number();
            if current_block_number > (start + length) {
                return Err(Error::TimeMatchOver)
            }
            // Check the deposit.
            // Assert or Error?
            let deposit = Self::env().transferred_value();
            if deposit < MIN_DEPOSIT {
                return Err(Error::NotEnoughDeposit)
            }
            // Create the betting match
            let betting_match = Match {
                start,
                length,
                team1,
                team2,
                result: None,
                bets: Default::default(),
                deposit,
            };
            // Check if match already exists by checking its specs hash.
            // How to create a hash of the object betting_match??
            // Store the match hash with its creator account.

            // Store the betting match in the list of open matches
            self.matches.insert(caller, &betting_match);
            // Emit an event.
            self.env().emit_event(MatchCreated {
                who: caller,
                team1: betting_match.team1,
                team2: betting_match.team2,
                start,
                length,
            });

            Ok(())
        }

        // payable accepts a payment (amount_to_bet).
        #[ink(message, payable)]
        pub fn bet(
            &mut self, 
            match_id: AccountId,
            result: MatchResult,
        ) -> Result<(), Error> {
            let caller = Self::env().caller();
            // Find the match that user wants to place the bet
            let mut match_to_bet = self.matches.get(&match_id).ok_or(Error::MatchDoesNotExist).unwrap();

            // Check if the Match Has Started (can't bet in a started match)
            let current_block_number = self.env().block_number();
            if current_block_number < match_to_bet.start {
                return Err(Error::MatchHasStarted)
            }
            let amount = Self::env().transferred_value();
            // Create the bet to be placed
            let bet = Bet {
                bettor: caller,
                amount,
                result: result.clone(),
            };
            // Check if the bet already exists
            if match_to_bet.bets.contains(&bet) { 
                return Err(Error::AlreadyBet);
            } else {
                match_to_bet.bets.push(bet);
                // Store the betting match in the list of open matches
                self.matches.insert(match_id, &match_to_bet);
                // Emit an event.
                self.env().emit_event(BetPlaced {
                    match_id,
                    who: caller,
                    amount,
                    result
                });
            }
            Ok(())
        }


        /// Simply checks if a match exists.
        #[ink(message)]
        pub fn exists_match(&self, owner: AccountId) -> bool {
            self.matches.contains(owner)
        }
    }
}
