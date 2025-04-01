use crate::helpers::clusters::yield_multiplier_weftv2::YMWeftV2ClusterService;
use crate::helpers::platform::{PlatformClusterService, PlatformService};
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
        .take_non_fungibles_from_worktop(weftv2.cdp, vec![cdp_id.clone()], "cdp_bucket")
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
}

#[test]
fn test_valid_account_open_with_non_empty_cdp() {
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
    let collateral = indexmap! { supply => dec!(1000) };
    let loan = indexmap! { debt => dec!(100) };
    let cdp_id = weftv2.mint(&mut runner, alice_account, Some(collateral), Some(loan), false);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_account_open_with_non_empty_invalid_supply_cdp() {
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
    let collateral = indexmap! { runner.faucet.usdc.address => dec!(1000) };
    let loan = indexmap! { debt => dec!(100) };
    let cdp_id = weftv2.mint(&mut runner, alice_account, Some(collateral), Some(loan), false);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_account_open_with_non_empty_invalid_debt_cdp() {
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
    let collateral = indexmap! { supply => dec!(1000) };
    let loan = indexmap! {  runner.faucet.usdc.address => dec!(100) };
    let cdp_id = weftv2.mint(&mut runner, alice_account, Some(collateral), Some(loan), false);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_account_open_with_non_empty_nft_cdp() {
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
    let collateral = indexmap! { supply => dec!(1000) };
    let loan = indexmap! { debt => dec!(100) };
    let cdp_id = weftv2.mint(&mut runner, alice_account, Some(collateral), Some(loan), true);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_account_double_open() {
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

    //] Act & Assert
    // Get a user badge
    platform.new_user(&mut runner, &alice_account);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();

    // Create new CDP
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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_close_account_without_open() {
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
    // Close account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .call_method_with_name_lookup(cluster.component, "close_account", |lookup| (lookup.proof("user_badge"),))
        .deposit_entire_worktop(alice_account.address);

    let receipt = runner.exec("close_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

/* ------------------- Link ------------------- */
#[test]
#[should_panic]
fn test_invalid_open_account_without_link() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();
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

    //] Act & Assert
    // Get a user badge
    platform.new_user(&mut runner, &alice_account);

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

    // println!("{:?}\n", receipt);
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_account_open_and_close_when_unlinked() {
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

    let receipt = runner.exec_and_dump("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Unlink cluster from platform
    platform.unlink(&mut runner, &owner_account, cluster.component);

    // Close account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .call_method_with_name_lookup(cluster.component, "close_account", |lookup| (lookup.proof("user_badge"),))
        .deposit_entire_worktop(alice_account.address);

    let receipt = runner.exec("close_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

/* ------------ Operating Services ------------ */
#[test]
#[should_panic]
fn test_invalid_account_open_when_platform_wide_service_disabled() {
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

    // Disable the OpenAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "update_service", manifest_args!(PlatformService::OpenAccount, false,));
    let receipt = runner.exec("update_platform_service", manifest, &owner_account, None);

    receipt.expect_commit_success();

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
}

#[test]
#[should_panic]
fn test_invalid_account_close_when_platform_wide_service_disabled() {
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

    let receipt = runner.exec_and_dump("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Disable the CloseAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            platform.component,
            "update_service",
            manifest_args!(PlatformService::CloseAccount, false,),
        );
    let receipt = runner.exec("update_platform_service", manifest, &owner_account, None);

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

#[test]
#[should_panic]
fn test_invalid_account_open_when_platform_specific_service_disabled() {
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

    // Disable the OpenAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            platform.component,
            "update_cluster_service",
            manifest_args!(cluster.component, PlatformClusterService::OpenAccount, false,),
        );
    let receipt = runner.exec_and_dump("update_platform_specific_service", manifest, &owner_account, None);

    receipt.expect_commit_success();

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
}

#[test]
#[should_panic]
fn test_invalid_account_close_when_platform_specific_service_disabled() {
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

    let receipt = runner.exec_and_dump("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Disable the CloseAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            platform.component,
            "update_cluster_service",
            manifest_args!(cluster.component, PlatformClusterService::CloseAccount, false,),
        );
    let receipt = runner.exec("update_platform_specific_service", manifest, &owner_account, None);

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

#[test]
#[should_panic]
fn test_invalid_account_open_when_cluster_service_disabled() {
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

    // Disable the OpenAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            cluster.component,
            "update_service",
            manifest_args!(YMWeftV2ClusterService::OpenAccount, false,),
        );
    let receipt = runner.exec_and_dump(
        "update_cluster_service",
        manifest,
        &owner_account,
        Some("clusters/yield_multiplier_weftv2"),
    );

    receipt.expect_commit_success();

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
}

#[test]
#[should_panic]
fn test_invalid_account_close_when_cluster_service_disabled() {
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

    // Disable the CloseAccount service
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account.address, platform.owner_badge, dec!(1))
        .call_method(
            cluster.component,
            "update_service",
            manifest_args!(YMWeftV2ClusterService::CloseAccount, false,),
        );
    let receipt = runner.exec(
        "update_cluster_service",
        manifest,
        &owner_account,
        Some("clusters/yield_multiplier_weftv2"),
    );

    receipt.expect_commit_success();

    // Close account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .call_method_with_name_lookup(cluster.component, "close_account", |lookup| (lookup.proof("user_badge"),))
        .deposit_entire_worktop(alice_account.address);

    let receipt = runner.exec("close_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}
