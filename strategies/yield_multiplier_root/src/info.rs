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
    pub fee_info: FeeInfo,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct FeeInfo {
    pub open: Decimal,
    pub close: Decimal,
    pub execute: Decimal,
}

/* ------------------ Account ----------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct AccountInfo {
    pub cdp_id: NonFungibleLocalId,
    pub supply_units: Decimal,
    pub debt_units: Decimal,
}
