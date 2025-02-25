use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

pub struct HelperPlatform {
    pub component_address: ComponentAddress,
    pub package_address: PackageAddress,
    pub owner_account: SimAccount,
    pub owner_badge_address: ResourceAddress,
}

impl HelperPlatform {
    pub fn new(ledger: &mut Ledger, owner: SimAccount) -> Self {
        let package_address = ledger.compile_and_publish(this_package!());

        // Call instantiation function
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(package_address, "Platform", "instantiate", manifest_args!())
            .deposit_batch(owner.address, ManifestExpression::EntireWorktop)
            .build();

        let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&owner.public_key)]);
        println!("{:?}\n", receipt);

        // Collect output
        let component = receipt.expect_commit_success().new_component_addresses()[0];
        let badge = receipt.expect_commit_success().new_resource_addresses()[0];

        println!("Component: {:?}\n", component);
        println!("Badge: {:?}\n", badge);

        // Return HelperPlatform
        Self {
            component_address: component,
            package_address,
            owner_account: owner.clone(),
            owner_badge_address: badge,
        }
    }
}
