use crate::helpers::{prelude::*, runner::TestRunner};
use scrypto_test::prelude::*;

#[test]
#[ignore = "test is redundant"]
fn test_platform_instantiates() {
    //] Arrange
    let mut runner = TestRunner::new();

    //] Act & Assert
    runner.platform;
}
