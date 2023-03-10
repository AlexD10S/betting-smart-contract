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
    fn default_works() {
        let betting = Betting::default();
        assert_eq!(betting.get(), false);
    }

    /// We test a simple use case of our contract.
    #[ink::test]
    fn it_works() {
        let mut betting = Betting::new(false);
        assert_eq!(betting.get(), false);
        betting.flip();
        assert_eq!(betting.get(), true);
    }
}