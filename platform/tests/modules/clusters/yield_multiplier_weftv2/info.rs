use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

/* ------------------ Cluster ----------------- */
#[test]
fn test_valid_get_cluster_info() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;

    // Instantiate a YieldMultiplierWeftCluster
    let weftv2 = MockWeftV2::new(&mut runner);
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

    //] Act & Assert
    // Link cluster to platform
    platform.link(&mut runner, &owner_account, cluster.component);

    // Get cluster info
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(cluster.component, "get_cluster_info", manifest_args!());
    let receipt = runner.exec_and_dump("get_cluster_info", manifest, &owner_account, Some("clusters/yield_multiplier_weftv2"));

    receipt.expect_commit_success();
}

#[test]
fn test_valid_get_cluster_info_unlinked() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;

    // Instantiate a YieldMultiplierWeftCluster
    let weftv2 = MockWeftV2::new(&mut runner);
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

    //] Act & Assert
    // Get cluster info
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(cluster.component, "get_cluster_info", manifest_args!());
    let receipt = runner.exec(
        "get_cluster_info_unlinked",
        manifest,
        &owner_account,
        Some("clusters/yield_multiplier_weftv2"),
    );

    receipt.expect_commit_success();
}

#[test]
fn test_a() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;

    // Instantiate a YieldMultiplierWeftCluster
    let weftv2 = MockWeftV2::new(&mut runner);
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

    //] Act & Assert
    // Link cluster to platform
    platform.link(&mut runner, &owner_account, cluster.component);

    // Get cluster info
    // let manifest = ManifestBuilder::new()
    //     .lock_fee_from_faucet()
    //     .call_method(cluster.component, "get_cluster_info", manifest_args!());
    // let receipt = runner.exec_and_dump("get_cluster_info", manifest, &owner_account, Some("clusters/yield_multiplier_weftv2"));

    // receipt.expect_commit_success();

    // let manifest = TransactionV2Builder
}

/* ------------------ Account ----------------- */
#[test]
#[ignore = "Requires mainnet"]
fn test_valid_get_account_info() {
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

    //] Act & Assert
    // Get an empty CDP
    let cdp_id = weftv2.mint_empty(&mut runner, alice_account);

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

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Get account info
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        cluster.component,
        "get_account_info",
        manifest_args!(NonFungibleLocalId::Integer(0.into()),),
    );
    let receipt = runner.exec_and_dump("get_account_info", manifest, &owner_account, Some("clusters/yield_multiplier_weftv2"));

    receipt.expect_commit_success();
}

#[test]
#[should_panic]
#[ignore = "Requires mainnet"]
fn test_invalid_get_account_info_without_account() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
    let owner_account = runner.owner_account;
    let alice_account = runner.alice_account;

    // Instantiate a YieldMultiplierWeftCluster
    let weftv2 = MockWeftV2::new(&mut runner);
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

    //] Act & Assert
    // Get account info
    let manifest = ManifestBuilder::new().lock_fee_from_faucet().call_method(
        cluster.component,
        "get_account_info",
        manifest_args!(NonFungibleLocalId::Integer(0.into()),),
    );
    let receipt = runner.exec_and_dump("get_account_info", manifest, &owner_account, Some("clusters/yield_multiplier_weftv2"));

    receipt.expect_commit_success();
}
