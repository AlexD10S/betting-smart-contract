#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

#[ink::contract]
mod betting {
    use ink::storage::Mapping;

    // Use BoundedVec?
    pub type TeamName = Vec<u8>;

    #[derive(scale::Decode, scale::Encode)]
    #[cfg_attr(
        feature = "std",
        derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
    )]
    pub enum MatchResult {
        Team1Victory,
        Team2Victory,
        Draw,
    }
    #[derive(scale::Decode, scale::Encode)]
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
    }

    impl Betting {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                matches: Default::default(),
            }
        }


        /// Simply returns the currentmapping of matches.
        #[ink(message)]
        pub fn exists_match(&self, owner: AccountId) -> bool {
            self.matches.contains(owner)
        }
    }
}
