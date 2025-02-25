use crate::helpers::platform::merge_path;
use platform::platform::platform_test::*;
use scrypto_test::prelude::*;

#[test]
fn simple_package_can_be_published() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();

    // Act & Assert
    let _ = ledger.compile_and_publish(this_package!());

    let path = merge_path("../strategies/yield_multiplier/weft/");

    let _ = ledger.compile_and_publish(path.as_str());
}

#[test]
fn test_platform_instantiates() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Act & Assert
    let package_address = ledger.compile_and_publish(this_package!());

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Platform", "instantiate", manifest_args!())
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    println!("{:?}\n", receipt);

    let component = receipt.expect_commit_success().new_component_addresses()[0];
    let badge = receipt.expect_commit_success().new_resource_addresses()[0];

    println!("Component: {:?}\n", component);
    println!("Badge: {:?}\n", badge);
}

#[test]
fn test_yield_multiplier_weft_cluster_instantiates() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Act & Assert
    let path = merge_path("../strategies/yield_multiplier/weft");
    let package_address = ledger.compile_and_publish(path.as_str());

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "YieldMultiplierClusterWeftV2", "instantiate", manifest_args!())
        .deposit_batch(account, ManifestExpression::EntireWorktop)
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&public_key)]);
    println!("{:?}\n", receipt);

    let component = receipt.expect_commit_success().new_component_addresses()[0];
    let badge = receipt.expect_commit_success().new_resource_addresses()[0];

    println!("Component: {:?}\n", component);
    println!("Badge: {:?}\n", badge);
}
