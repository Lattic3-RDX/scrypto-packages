use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

#[test]
fn test_valid_mint_new_user_badge() {
    //] Arrange
    let (mut runner, platform) = Runner::new_base();

    //] Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(platform.component, "new_user", manifest_args!())
        .deposit_batch(runner.alice_account.address, ManifestExpression::EntireWorktop)
        .build();

    //] Assert
    let receipt = runner.ledger.execute_manifest(manifest, vec![runner.alice_account.global_id()]);

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    let alice_resources = runner.ledger.get_component_resources(runner.alice_account.address);

    assert_eq!(alice_resources.len(), 1);
}
