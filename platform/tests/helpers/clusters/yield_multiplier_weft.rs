use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

//] ------------------ Factory ----------------- */
#[derive(Debug, Clone, Copy)]
pub struct YMWeftClusterFactory {
    pub package_address: PackageAddress,
}

impl YMWeftClusterFactory {
    pub fn new(ledger: &mut Ledger) -> Self {
        let path = merge_path("../strategies/yield_multiplier_weft");
        let package_address = ledger.compile_and_publish(path);

        Self { package_address }
    }

    pub fn instantiate(
        &self,
        runner: &mut Runner,
        // Authorisation
        owner_rule: AccessRule,
        // Link
        platform_address: ComponentAddress,
        link_address: ResourceAddress,
        user_badge_address: ResourceAddress,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        // Integration
        cdp_address: ResourceAddress,
    ) -> YMWeftCluster {
        // Call instantiation function
        #[rustfmt::skip]
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                self.package_address,
                "YieldMultiplierWeftCluster", "instantiate",
                manifest_args!(
                    owner_rule,
                    platform_address, link_address, user_badge_address,
                    supply, debt,
                    cdp_address
                ),
            )
            .build();

        let receipt = runner
            .ledger
            .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&runner.owner_account.public_key)]);
        // println!("{:?}\n", receipt);

        // Collect output
        let component_address = receipt.expect_commit_success().new_component_addresses()[0];
        let execution_terms = receipt.expect_commit_success().new_resource_addresses()[0];

        println!("Execution Terms: {:?}\n", execution_terms,);

        // Return HelperYMWeftCluster
        YMWeftCluster {
            component_address,
            platform_address,
            link_address,
            user_badge_address,
            supply,
            debt,
            cdp_address,
        }
    }
}

//] ------------------ Cluster ----------------- */
#[derive(Debug, Clone, Copy)]
pub struct YMWeftCluster {
    // General
    pub component_address: ComponentAddress,
    // Linking
    pub platform_address: ComponentAddress,
    pub link_address: ResourceAddress,
    pub user_badge_address: ResourceAddress,
    // Cluster
    pub supply: ResourceAddress,
    pub debt: ResourceAddress,
    // Weft integration
    pub cdp_address: ResourceAddress,
}
