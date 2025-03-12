use crate::helpers::{platform::PlatformService, prelude::*};
use scrypto_test::prelude::*;

#[test]
fn test_valid_mint_new_user_badge() {
    //] Arrange
    let (mut runner, platform) = Runner::new_base();
    let alice_account = runner.alice_account;

    //] Act & Assert
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(platform.component, "new_user", manifest_args!())
        .deposit_batch(alice_account.address, ManifestExpression::EntireWorktop);
    let receipt = runner.exec_and_dump("new_user", manifest, &alice_account, None);

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    let alice_resources = runner.ledger.get_component_resources(alice_account.address);

    assert_eq!(alice_resources.len(), 1);
}

/* ------------ Operating Services ------------ */
#[test]
#[should_panic]
fn test_invalid_new_user_when_service_disabled() {
    //] Arrange
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;
    let alice_account = runner.alice_account;

    //] Act & Assert
    // Disable the MintBadge service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "update_service", manifest_args!(PlatformService::MintBadge, false,));
    let receipt = runner.exec_and_dump("update_platform_service", manifest, &owner_account, None);

    receipt.expect_commit_success();

    // Attempt to mint a new user badge
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(platform.component, "new_user", manifest_args!())
        .deposit_batch(alice_account.address, ManifestExpression::EntireWorktop);
    let receipt = runner.exec_and_dump("new_user", manifest, &alice_account, None);

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    let alice_resources = runner.ledger.get_component_resources(alice_account.address);

    assert_eq!(alice_resources.len(), 1);
}
