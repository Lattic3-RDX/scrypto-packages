use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor, Debug, Clone)]
pub struct ExecutionTerms {
    pub user_local_id: NonFungibleLocalId,
}

impl ExecutionTerms {
    pub fn new(user_local_id: NonFungibleLocalId) -> Self {
        ExecutionTerms { user_local_id }
    }
}
