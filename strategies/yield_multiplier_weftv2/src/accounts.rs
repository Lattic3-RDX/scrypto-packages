/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::utils::now;

/* ------------------- User ------------------- */
#[derive(ScryptoSbor, Debug)]
pub struct AccountData {
    pub cdp_vault: NonFungibleVault,
    // pub fee_vault: FungibleVault,
    pub updated_at: i64,
    pub supply_delta: Decimal,
    pub debt_delta: Decimal,
}

impl AccountData {
    pub fn new(cdp_vault: NonFungibleVault, supply_delta: Decimal, debt_delta: Decimal) -> Self {
        Self {
            cdp_vault,
            // fee_vault,
            updated_at: now(),
            supply_delta,
            debt_delta,
        }
    }
}
