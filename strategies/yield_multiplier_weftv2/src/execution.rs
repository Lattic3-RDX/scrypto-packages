use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor, Debug, Clone)]
pub struct ExecutionTerms {
    pub cdp_id: NonFungibleLocalId,
    pub user_local_id: NonFungibleLocalId,
    // pub cdp_liquidity: Decimal,
}

impl ExecutionTerms {
    pub fn new(
        cdp_id: NonFungibleLocalId,
        user_local_id: NonFungibleLocalId,
        // cdp_liquidity: Decimal
    ) -> Self {
        ExecutionTerms {
            cdp_id,
            user_local_id,
            // cdp_liquidity
        }
    }
}
