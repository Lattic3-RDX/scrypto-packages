use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

#[test]
fn test_valid_link_and_unlink() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();

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
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(runner.owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "link_cluster", manifest_args!(cluster.component,))
        .build();

    let receipt = runner.ledger.execute_manifest(manifest, vec![runner.owner_account.global_id()]);
    receipt.expect_commit_success();

    // Unlink cluster from platform
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(runner.owner_account.address, platform.owner_badge, dec!(1))
        .call_method(platform.component, "unlink_cluster", manifest_args!(cluster.component,))
        .build();

    let receipt = runner.ledger.execute_manifest(manifest, vec![runner.owner_account.global_id()]);
    receipt.expect_commit_success();
}
