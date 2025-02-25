use crate::helpers::{prelude::*, runner::TestRunner};
use scrypto_test::prelude::*;

#[test]
fn test_platform_with_weft_instantiates() {
    TestRunner::new_weft();
}
