use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use indexmap::indexmap;
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
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),))
        .take_all_from_worktop(cluster.execution_terms, "execution_term_bucket")
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "end_execution", |lookup| {
            (
                lookup.proof("user_badge_2"),
                lookup.bucket("cdp_bucket"),
                lookup.bucket("execution_term_bucket"),
            )
        });

    let receipt = runner.exec_and_dump("execute", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

#[test]
fn test_valid_execute_with_change() {
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
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        // Setup authorisation
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        // Start execution and collect output CDP and terms
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),))
        .take_all_from_worktop(cluster.execution_terms, "execution_term_bucket")
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        // Update CDP
        .update_non_fungible_data(
            weftv2.cdp,
            cdp_id.clone(),
            "collaterals",
            MockWeftV2::map_collaterals(indexmap! { supply => dec!(1000) }),
        )
        .update_non_fungible_data(
            weftv2.cdp,
            cdp_id.clone(),
            "loans",
            MockWeftV2::map_loans(indexmap! { debt => dec!(100) }),
        )
        // End execution
        .call_method_with_name_lookup(cluster.component, "end_execution", |lookup| {
            (
                lookup.proof("user_badge_2"),
                lookup.bucket("cdp_bucket"),
                lookup.bucket("execution_term_bucket"),
            )
        });

    let receipt = runner.exec("execute_valid_change", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_execute_with_change() {
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
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        // Setup authorisation
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        // Start execution and collect output CDP and terms
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),))
        .take_all_from_worktop(cluster.execution_terms, "execution_term_bucket")
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        // Update CDP
        .update_non_fungible_data(
            weftv2.cdp,
            cdp_id.clone(),
            "collaterals",
            MockWeftV2::map_collaterals(indexmap! { debt => dec!(1000) }),
        )
        .update_non_fungible_data(
            weftv2.cdp,
            cdp_id.clone(),
            "loans",
            MockWeftV2::map_loans(indexmap! { supply => dec!(100) }),
        )
        // End execution
        .call_method_with_name_lookup(cluster.component, "end_execution", |lookup| {
            (
                lookup.proof("user_badge_2"),
                lookup.bucket("cdp_bucket"),
                lookup.bucket("execution_term_bucket"),
            )
        });

    let receipt = runner.exec(
        "execute_invalid_change",
        manifest,
        &alice_account,
        Some("clusters/yield_multiplier_weftv2"),
    );
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_execute_without_ending() {
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
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),));

    let receipt = runner.exec("execute_no_end", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

#[test]
#[should_panic]
fn test_invalid_execute_different_cdps() {
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
    let other_cdp_id = weftv2.mint_empty(&mut runner, alice_account);

    //] Act & Assert
    // Open an account
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .withdraw_non_fungibles_from_account(alice_account.address, weftv2.cdp, vec![cdp_id.clone()])
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),))
        .take_all_from_worktop(cluster.execution_terms, "execution_term_bucket")
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .deposit(alice_account.address, "cdp_bucket")
        .withdraw_non_fungibles_from_account(alice_account.address, weftv2.cdp, vec![other_cdp_id.clone()])
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket_2")
        .call_method_with_name_lookup(cluster.component, "end_execution", |lookup| {
            (
                lookup.proof("user_badge_2"),
                lookup.bucket("cdp_bucket_2"),
                lookup.bucket("execution_term_bucket"),
            )
        });

    let receipt = runner.exec_and_dump("execute", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

/* ------------------- Link ------------------- */
#[test]
#[should_panic]
fn test_invalid_execute_without_link() {
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
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "open_account", |lookup| {
            (lookup.proof("user_badge"), lookup.bucket("cdp_bucket"))
        });

    let receipt = runner.exec("open_account", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();

    //] Act & Assert
    // Execute a transaction
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_non_fungibles(alice_account.address, platform.user_badge, vec![NonFungibleLocalId::Integer(0.into())])
        .pop_from_auth_zone("user_badge")
        .clone_proof("user_badge", "user_badge_2")
        .call_method_with_name_lookup(cluster.component, "start_execution", |lookup| (lookup.proof("user_badge"),))
        .take_all_from_worktop(cluster.execution_terms, "execution_term_bucket")
        .take_all_from_worktop(weftv2.cdp, "cdp_bucket")
        .call_method_with_name_lookup(cluster.component, "end_execution", |lookup| {
            (
                lookup.proof("user_badge_2"),
                lookup.bucket("cdp_bucket"),
                lookup.bucket("execution_term_bucket"),
            )
        });

    let receipt = runner.exec("execute", manifest, &alice_account, Some("clusters/yield_multiplier_weftv2"));
    receipt.expect_commit_success();
}

/* ------------ Operating Services ------------ */
