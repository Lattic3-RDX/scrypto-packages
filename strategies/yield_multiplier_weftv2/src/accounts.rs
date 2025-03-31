/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
// use shared::utils::now;

/* ------------------- User ------------------- */
#[derive(ScryptoSbor, Debug)]
pub struct AccountData {
    pub cdp_vault: NonFungibleVault,
    // pub fee_vault: FungibleVault,
}

impl AccountData {
    pub fn new(cdp_vault: NonFungibleVault) -> Self {
        Self {
            cdp_vault,
            // fee_vault,
            // updated_at: now(),
            // supply_delta,
            // debt_delta,
        }
    }
}
