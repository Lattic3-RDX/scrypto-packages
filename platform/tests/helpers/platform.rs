use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct PlatformFactory {
    pub package: PackageAddress,
}

impl PlatformFactory {
    pub fn new(ledger: &mut Ledger) -> Self {
        let package = ledger.compile_and_publish(this_package!());

        PlatformFactory { package }
    }

    pub fn instantiate(&self, ledger: &mut Ledger, owner: SimAccount) -> Platform {
        // Call instantiation function
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(self.package, "Platform", "instantiate", manifest_args!())
            .deposit_batch(owner.address, ManifestExpression::EntireWorktop)
            .build();

        let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&owner.public_key)]);
        // println!("{:?}\n", receipt);

        // Collect output
        let component = receipt.expect_commit_success().new_component_addresses()[0];
        let owner_badge = receipt.expect_commit_success().new_resource_addresses()[0];
        let user_badge = receipt.expect_commit_success().new_resource_addresses()[1];
        let link_badge = receipt.expect_commit_success().new_resource_addresses()[2];

        println!("Component: {:?}\n", component);
        println!("Owner Badge: {:?}\n", owner_badge);
        println!("User Badge: {:?}\n", user_badge);
        println!("Link Badge: {:?}\n", link_badge);

        // Return HelperPlatform
        Platform { component, owner_account: owner.clone(), owner_badge, user_badge, link_badge }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Platform {
    pub component: ComponentAddress,
    pub owner_account: SimAccount,
    pub owner_badge: ResourceAddress,
    pub user_badge: ResourceAddress,
    pub link_badge: ResourceAddress,
}
