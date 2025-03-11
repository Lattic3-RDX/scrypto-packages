use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor, Debug, Clone)]
pub struct ExecutionTerms {
    pub cdp_id: NonFungibleLocalId,
    pub user_local_id: NonFungibleLocalId,
}

impl ExecutionTerms {
    pub fn new(cdp_id: NonFungibleLocalId, user_local_id: NonFungibleLocalId) -> Self {
        ExecutionTerms { cdp_id, user_local_id }
    }
}
