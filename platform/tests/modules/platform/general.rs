use crate::helpers::prelude::*;
use crate::helpers::{clusters::yield_multiplier_weftv2::YMWeftV2ClusterFactory, integrations::weftv2::MockWeftV2};
use scrypto_test::prelude::*;

#[test]
#[ignore = "placeholder"]
fn test_platform_with_weft_instantiates() {
    //] Arrange
    let (mut runner, platform) = Runner::new_base();
    let weft = MockWeftV2::new(&mut runner);

    //] Act & Assert
    let ym_cluster_weft_factory = YMWeftV2ClusterFactory::new(&mut runner.ledger);

    let owner_rule = rule!(require(platform.owner_badge));
    let supply = runner.faucet.usdt.address;
    let debt = runner.faucet.xwbtc.address;

    ym_cluster_weft_factory.instantiate(
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
