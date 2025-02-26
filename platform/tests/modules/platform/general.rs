use crate::helpers::{clusters::yield_multiplier_weft::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2, runner::Runner};
use scrypto_test::prelude::*;

#[test]
fn test_platform_with_weft_instantiates() {
    //] Arrange
    let (mut runner, platform) = Runner::new_base();
    let weftv2 = MockWeftV2::new(&mut runner);

    //] Act & Assert
    let ym_cluster_weftv2_factory = YMWeftV2ClusterFactory::new(&mut runner.ledger);

    let owner_rule = rule!(require(platform.owner_badge));
    let supply = runner.faucet.usdt.address;
    let debt = runner.faucet.xwbtc.address;

    ym_cluster_weftv2_factory.instantiate(
        &mut runner,
        owner_rule,
        platform.component,
        platform.link_badge,
        platform.user_badge,
        supply,
        debt,
        weftv2.cdp,
    );
}
