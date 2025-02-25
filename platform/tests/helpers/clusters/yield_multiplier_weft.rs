use crate::helpers::platform::{merge_path, Ledger};
use scrypto_test::prelude::*;

pub struct HelperYieldMultiplierClusterWeftV2 {
    // General
    pub component_address: ComponentAddress,
    // Linking
    pub platform_address: ComponentAddress,
    pub link_address: ResourceAddress,
    pub user_badge_address: ResourceAddress,
    // Cluster
    pub supply: ResourceAddress,
    pub debt: ResourceAddress,
    // WeftV2 integration
    pub cdp_address: ResourceAddress,
}

impl HelperYieldMultiplierClusterWeftV2 {
    pub fn publish(ledger: &mut Ledger) -> PackageAddress {
        let path = merge_path("../strategies/yield_multiplier/weft");
        ledger.compile_and_publish(path)
    }

    pub fn new(
        // Testing
        ledger: &mut Ledger,
        package_address: PackageAddress,
        // Authorisation
        owner_proof: FungibleProof,
        // Link
        platform_address: ComponentAddress,
        link_resource: ResourceAddress,
        user_badge_address: ResourceAddress,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        // Integration
        cdp_resource: ResourceAddress,
    ) {
    }
}
