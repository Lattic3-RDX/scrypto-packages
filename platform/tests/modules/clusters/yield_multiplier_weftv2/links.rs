use crate::helpers::clusters::yield_multiplier_weftv2::YMWeftV2ClusterService;
use crate::helpers::platform::PlatformService;
use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

#[test]
fn test_valid_link_and_unlink() {
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

    // Unlink cluster from platform
    platform.unlink(&mut runner, &owner_account, cluster.component);
}

#[test]
#[should_panic]
fn test_invalid_double_link() {
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

    // Link cluster to platform
    platform.link(&mut runner, &owner_account, cluster.component);

    //] Act & Assert
    // Link again
    platform.link(&mut runner, &owner_account, cluster.component);
}

#[test]
#[should_panic]
fn test_invalid_link_from_other() {
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
    // Create second platform
    let other_platform = runner.platform_factory.instantiate(&mut runner.ledger, owner_account);

    // Link cluster to other platform
    other_platform.link(&mut runner, &owner_account, cluster.component);
}

/* ------------ Operating Services ------------ */
#[test]
#[should_panic]
fn test_invalid_link_when_platform_service_disabled() {
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
    // Disable the LinkCluster service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "update_service", manifest_args!(PlatformService::LinkCluster, false,));
    let receipt = runner.exec("update_platform_service", manifest, &owner_account, None);

    receipt.expect_commit_success();

    // Link cluster to platform
    platform.link(&mut runner, &owner_account, cluster.component);
}

#[test]
#[should_panic]
fn test_invalid_unlink_when_platform_service_disabled() {
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

    // Disable the UnlinkCluster service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            platform.component,
            "update_service",
            manifest_args!(PlatformService::UnlinkCluster, false,),
        );
    let receipt = runner.exec("update_platform_service", manifest, &owner_account, None);

    receipt.expect_commit_success();

    // Unlink cluster from platform
    platform.unlink(&mut runner, &owner_account, cluster.component);
}

#[test]
#[should_panic]
fn test_invalid_link_when_cluster_service_disabled() {
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
    // Disable the Link service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(cluster.component, "update_service", manifest_args!(YMWeftV2ClusterService::Link, false,));
    let receipt = runner.exec(
        "update_cluster_service",
        manifest,
        &owner_account,
        Some("clusters/yield_multiplier_weftv2"),
    );

    receipt.expect_commit_success();

    // Link cluster to platform
    platform.link(&mut runner, &owner_account, cluster.component);
}

#[test]
#[should_panic]
#[ignore = "YMWeftV2ClusterService::Unlink unused"]
fn test_invalid_unlink_when_cluster_service_disabled() {
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

    // Disable the Unlink service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            cluster.component,
            "update_service",
            manifest_args!(YMWeftV2ClusterService::Unlink, false,),
        );
    let receipt = runner.exec(
        "update_cluster_service",
        manifest,
        &owner_account,
        Some("clusters/yield_multiplier_weftv2"),
    );

    receipt.expect_commit_success();

    // Unlink cluster from platform
    platform.unlink(&mut runner, &owner_account, cluster.component);
}
