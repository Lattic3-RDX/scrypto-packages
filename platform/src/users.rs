use scrypto::prelude::{indexmap::IndexSet, *};

#[derive(NonFungibleData, ScryptoSbor)]
pub struct User {
    #[mutable]
    pub open_accounts_in: IndexSet<ComponentAddress>,
}

impl User {
    pub fn new() -> Self {
        Self { open_accounts_in: IndexSet::new() }
    }
}
