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
        pub bettor: AccountId,
        /// Bet amount.
        pub amount: Balance,
        /// Result predicted.
        pub result: MatchResult,
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
        pub bets: Vec<Bet>,
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
        /// Owner of the Smart Contract (sudo)
        owner: AccountId,
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
    /// A match result has been set. [matchId, result]
    #[ink(event)]
    pub struct MatchResultSet {
        #[ink(topic)]
        match_id: AccountId, 
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
        /// Only owner of the smart contract can make this call
        BadOrigin,
        /// No allowing set the result if the match not over
        TimeMatchNotOver,
    }

    impl Betting {
        #[ink(constructor)]
        pub fn new() -> Self {
            let owner = Self::env().caller();
            Self {
                matches: Default::default(),
                owner
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
            let mut match_to_bet = match self.matches.take(&match_id) {
                Some(match_from_storage) => match_from_storage,
                None => return Err(Error::MatchDoesNotExist)
            };

            // Check if the Match Has Started (can't bet in a started match)
            let current_block_number = self.env().block_number();
            if current_block_number > match_to_bet.start {
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


        /// Set the result of an existing match.
        /// The dispatch origin for this call must be the owner.
        /// Get root of the node?? like ensure_root(origin)?;
        #[ink(message, payable)]
        pub fn set_result(
            &mut self, 
            match_id: AccountId,
            result: MatchResult,
        ) -> Result<(), Error> {
            let caller = Self::env().caller();
            // Only owner of the SC can call this message.
            if caller != self.owner {
                return Err(Error::BadOrigin);
            }
            //Find the match where owner wants to set the result
            let mut match_to_set_result = match self.matches.take(&match_id) {
                Some(match_from_storage) => match_from_storage,
                None => return Err(Error::MatchDoesNotExist)
            };
            // Check if start and length are valid
            let current_block_number = self.env().block_number();
            if current_block_number <= (match_to_set_result.start + match_to_set_result.length) {
                return Err(Error::TimeMatchNotOver)
            }
            //set the result
            match_to_set_result.result = Some(result.clone());
            // Store the betting match in the list of open matches
            self.matches.insert(match_id, &match_to_set_result);
            // Emit an event.
            self.env().emit_event(MatchResultSet {
                match_id,
                result
            });
            
            Ok(())
        }


        /// Simply checks if a match exists.
        #[ink(message)]
        pub fn exists_match(&self, owner: AccountId) -> bool {
            self.matches.contains(owner)
        }
        #[ink(message)]
        pub fn get_match(&self, owner: AccountId) -> Option<Match> {
            self.matches.get(owner)
        }
    }
}
