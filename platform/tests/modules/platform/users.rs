use crate::helpers::prelude::*;
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
