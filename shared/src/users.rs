// Modules
use crate::utils::now;
// Libraries
use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug)]
pub enum UserBadge {
    Raw(NonFungibleProof),
    Valid(NonFungibleLocalId),
}

#[derive(NonFungibleData, ScryptoSbor)]
pub struct User {
    // #[immutable]
    pub minted_at: Instant,
    #[mutable]
    pub accounts_in: IndexSet<ComponentAddress>,
    #[mutable]
    pub open: u64,
}

impl User {
    pub fn new() -> Self {
        Self { minted_at: Instant::new(now()), accounts_in: IndexSet::new(), open: 0 }
    }

    pub fn add_account(&mut self, cluster_address: ComponentAddress) {
        assert!(self.accounts_in.get(&cluster_address).is_none(), "Account already exists");
        assert!(self.open < u64::MAX, "Cannot open more accounts; at U64 MAX");

        self.accounts_in.insert(cluster_address);
        self.open += 1;
    }

    pub fn remove_account(&mut self, cluster_address: ComponentAddress) {
        assert!(self.accounts_in.get(&cluster_address).is_some(), "Account does not exist");
        assert!(self.open > 0, "Invalid state: account exists but open is at 0");

        self.accounts_in.shift_remove(&cluster_address);
        self.open -= 1;
    }
}
