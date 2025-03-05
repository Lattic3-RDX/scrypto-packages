use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

#[test]
fn test_valid_account_open_and_close() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;
    let alice_account = runner.alice_account;

    // Instantiate a YieldMultiplierWeftCluster
    println!("Instantiating MockWeftV2");
    let mut weftv2 = MockWeftV2::new(&mut runner);
    let ym_weftv2_cluster_factory = YMWeftV2ClusterFactory::new(&mut runner.ledger);

    let owner_rule = rule!(require(platform.owner_badge));
    let supply = runner.faucet.usdt.address;
    let debt = runner.faucet.xwbtc.address;

    println!("Instantiating YieldMultiplierWeftCluster");
    let cluster = ym_weftv2_cluster_factory.instantiate(
        &mut runner,
        owner_rule,
        platform.component,
        platform.link_badge,
        platform.user_badge,
        supply,
        debt,
        weftv2.cdp,
    );

    // Link cluster to platform
    println!("Linking cluster to platform");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "link_cluster", manifest_args!(cluster.component,))
        .build();

    let receipt = runner.ledger.execute_manifest(manifest, vec![owner_account.global_id()]);
    receipt.expect_commit_success();

    //] Act & Assert
    // Get a user badge
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(platform.component, "new_user", manifest_args!())
        .deposit_batch(alice_account.address, ManifestExpression::EntireWorktop)
        .build();

    let receipt = runner.ledger.execute_manifest(manifest, vec![alice_account.global_id()]);
    receipt.expect_commit_success();

    // Get an empty CDP
    let cdp_id = weftv2.mint_empty(&mut runner, alice_account);

    // Open an account
    println!("Opening account");
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .withdraw_non_fungibles_from_account(alice_account.address, weftv2.cdp, vec![cdp_id.clone()])
        .take_non_fungibles_from_worktop(weftv2.cdp, vec![cdp_id], "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        })
        .build();

    let receipt = runner.ledger.execute_manifest(manifest, vec![alice_account.global_id()]);

    println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}
