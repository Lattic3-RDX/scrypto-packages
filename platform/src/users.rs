use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
pub struct User {
    // #[immutable]
    pub minted_at: Instant,
    #[mutable]
    pub accounts_in: KeyValueStore<ComponentAddress, ()>,
    #[mutable]
    pub open: u64,
}

impl User {
    pub fn new() -> Self {
        let seconds_since_unix_epoch = Clock::current_time(TimePrecisionV2::Second).seconds_since_unix_epoch;

        Self {
            minted_at: Instant::new(seconds_since_unix_epoch),
            accounts_in: KeyValueStore::new(),
            open: 0,
        }
    }

    pub fn add_account(&mut self, cluster_address: ComponentAddress) {
        assert!(self.accounts_in.get(&cluster_address).is_none(), "Account already existing");
        assert!(self.open < u64::MAX, "Cannot open more accounts; at U64 MAX");

        self.accounts_in.insert(cluster_address, ());
        self.open += 1;
    }

    pub fn remove_account(&mut self, cluster_address: ComponentAddress) {
        assert!(self.accounts_in.get(&cluster_address).is_some(), "Account does not exist");
        assert!(self.open > 0, "Invalid state: account exists but open is at 0");

        self.accounts_in.remove(&cluster_address);
        self.open -= 1;
    }
}
