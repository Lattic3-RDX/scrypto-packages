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
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone)]
pub struct EventClusterInfo {
    pub info: ClusterInfo,
}

/* ------------------ Account ----------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct AccountInfo {
    pub cdp_id: NonFungibleLocalId,
    pub supply: Decimal,
    pub debt: Decimal,
}

#[derive(ScryptoSbor, ScryptoEvent, Debug, Clone)]
pub struct EventAccountInfo {
    pub info: AccountInfo,
}
