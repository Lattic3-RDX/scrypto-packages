/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* ------------------- User ------------------- */
#[derive(ScryptoSbor, Debug)]
pub struct AccountData {
    pub cdp_vault: NonFungibleVault,
}

impl AccountData {
    pub fn new(cdp_vault: NonFungibleVault) -> Self {
        Self { cdp_vault }
    }
}
