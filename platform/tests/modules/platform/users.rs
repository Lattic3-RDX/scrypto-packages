use crate::helpers::runner::TestRunner;
use scrypto_test::prelude::*;

#[test]
fn test_valid_new_user_badge() {
    //] Arrange
    let mut runner = TestRunner::new();

    //] Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(runner.platform.component_address, "new_user", manifest_args!())
        .deposit_batch(runner.alice_account.address, ManifestExpression::EntireWorktop)
        .build();

    //] Assert
    let receipt = runner.ledger.execute_manifest(manifest, vec![runner.alice_account.global_id()]);

    println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    // let alice_resources = runner.ledger.get_component_resources(runner.alice_account.address);

    // assert_eq!(alice_resources.len(), 1);
}

#[test]
#[ignore = "Placeholder"]
fn test_new_user_when_service_disabled() {}

#[test]
#[ignore = "Placeholder"]
fn test_valid_cluster_badge_update() {}

#[test]
#[ignore = "Placeholder"]
fn test_cluster_badge_update_when_platform_service_disabled() {}

#[test]
#[ignore = "Placeholder"]
fn test_cluster_badge_update_when_cluster_service_disabled() {}
