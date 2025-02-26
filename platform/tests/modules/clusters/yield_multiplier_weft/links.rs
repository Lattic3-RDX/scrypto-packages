use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weft::YMWeftClusterFactory, integrations::weft::MockWeft};
use scrypto_test::prelude::*;

#[test]
fn test_valid_link_and_unlink() {
    //] Arrange
    // Create a test runner and platform
    let (mut runner, platform) = Runner::new_base();

    // Instantiate a YieldMultiplierWeftCluster
    let weft = MockWeft::new(&mut runner);
    let ym_weft_cluster_factory = YMWeftClusterFactory::new(&mut runner.ledger);

    //] Act & Assert

    let owner_rule = rule!(require(platform.owner_badge));
    let supply = runner.faucet.usdt.address;
    let debt = runner.faucet.xwbtc.address;

    ym_weft_cluster_factory.instantiate(
        &mut runner,
        owner_rule,
        platform.component,
        platform.link_badge,
        platform.user_badge,
        supply,
        debt,
        weft.cdp,
    );
}
