use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

#[test]
fn test_valid_execute_empty() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;
    let alice_account = runner.alice_account;

    // Instantiate a YieldMultiplierWeftCluster
    let mut weftv2 = MockWeftV2::new(&mut runner);
    let ym_weftv2_cluster_factory = YMWeftV2ClusterFactory::new(&mut runner.ledger);

    let owner_rule = rule!(require(platform.owner_badge));
    let supply = runner.faucet.usdt.address;
    let debt = runner.faucet.xwbtc.address;

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
    platform.link(&mut runner, &owner_account, cluster.component);

    // Get a user badge
    platform.new_user(&mut runner, &alice_account);

    // Get an empty CDP
    let cdp_id = weftv2.mint_empty(&mut runner, alice_account);

    //] Act & Assert
    // Open an account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .withdraw_non_fungibles_from_account(alice_account.address, weftv2.cdp, vec![cdp_id.clone()])
        .take_non_fungibles_from_worktop(weftv2.cdp, vec![cdp_id], "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec_and_dump("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Close account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .call_method_with_name_lookup(cluster.component, "close_account", |lookup| (lookup.proof("user_badge"),))
        .deposit_entire_worktop(alice_account.address);

    let receipt = runner.exec_and_dump("close_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}
