/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* ------------------ Cluster ----------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterInfo {
    pub platform_address: ComponentAddress,
    pub cluster_address: ComponentAddress,
    pub linked: bool,
    pub supply_res: ResourceAddress,
    pub debt_res: ResourceAddress,
    pub account_count: u64,
    pub execution_term_manager: NonFungibleResourceManager,
}

/* ------------------ Account ----------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct AccountInfo {
    pub cdp_id: NonFungibleLocalId,
    pub supply: Decimal,
    pub supply_value: Decimal,
    pub debt: Decimal,
    pub debt_value: Decimal,
    pub health: Decimal,
    pub platform_fee_due: Decimal,
}
