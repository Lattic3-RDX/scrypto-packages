use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

//] ------------------ Factory ----------------- */
#[derive(Debug, Clone, Copy)]
pub struct YMWeftV2ClusterFactory {
    pub package_address: PackageAddress,
}

impl YMWeftV2ClusterFactory {
    pub fn new(ledger: &mut Ledger) -> Self {
        let path = merge_path("../strategies/yield_multiplier_weftv2");
        let package_address = ledger.compile_and_publish(path);

        Self { package_address }
    }

    pub fn instantiate(
        &self,
        runner: &mut Runner,
        // Authorisation
        owner_rule: AccessRule,
        // Link
        platform: ComponentAddress,
        link_badge: ResourceAddress,
        user_badge: ResourceAddress,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        // Integration
        cdp: ResourceAddress,
    ) -> YMWeftV2Cluster {
        let owner_account = runner.owner_account;

        // Call instantiation function
        #[rustfmt::skip]
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                self.package_address,
                "YieldMultiplierWeftV2Cluster", "instantiate",
                manifest_args!(
                    owner_rule,
                    platform, link_badge, user_badge,
                    supply, debt,
                    platform, cdp
                ),
            );

        // let receipt = runner
        // .ledger
        // .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&runner.owner_account.public_key)]);
        // println!("{:?}\n", receipt);
        let receipt = runner.exec_and_dump("instantiate", manifest, &owner_account, Some("clusters/yield_multiplier_weftv2"));

        // Collect output
        let component = receipt.expect_commit_success().new_component_addresses()[0];
        let execution_terms = receipt.expect_commit_success().new_resource_addresses()[0];

        println!("Execution Terms: {:?}\n", execution_terms,);

        // Return YMWeftV2Cluster
        YMWeftV2Cluster { component, platform, link_badge, user_badge, supply, debt, execution_terms, cdp }
    }
}

//] ------------------ Cluster ----------------- */
#[derive(Debug, Clone, Copy)]
pub struct YMWeftV2Cluster {
    // General
    pub component: ComponentAddress,
    // Linking
    pub platform: ComponentAddress,
    pub link_badge: ResourceAddress,
    pub user_badge: ResourceAddress,
    // Cluster
    pub supply: ResourceAddress,
    pub debt: ResourceAddress,
    pub execution_terms: ResourceAddress,
    // WeftV2 integration
    pub cdp: ResourceAddress,
}

#[derive(ScryptoSbor, Debug, Clone, Copy, ManifestSbor)]
pub enum YMWeftV2ClusterService {
    OpenAccount,
    CloseAccount,
    Execute,
    Link,
    // Unlink,
    CallLinked,
}
