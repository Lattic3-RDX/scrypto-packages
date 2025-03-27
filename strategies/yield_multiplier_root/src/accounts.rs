/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::utils::now;

/* ------------------- User ------------------- */
#[derive(ScryptoSbor, Debug)]
pub struct AccountData {
    pub cdp_vault: NonFungibleVault,
    // pub fee_vault: FungibleVault,
    pub updated_at: i64,
    pub initial_liquidity: Decimal,
}

impl AccountData {
    pub fn new(cdp_vault: NonFungibleVault, initial_liquidity: Decimal) -> Self {
        Self {
            cdp_vault,
            // fee_vault,
            updated_at: now(),
            initial_liquidity,
        }
    }
}
