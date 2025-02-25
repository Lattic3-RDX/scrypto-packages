use crate::helpers::{integrations::weft::HelperWeftV2, platform::HelperPlatform, prelude::*};
use scrypto_test::prelude::*;

pub struct TestRunner {
    // Simulation
    pub ledger: Ledger,
    // Accounts
    pub owner_account: SimAccount,
    pub alice_account: SimAccount,
    pub bob_account: SimAccount,
    // Components
    pub platform: HelperPlatform,
    // Integrations
    pub weftv2: Option<HelperWeftV2>,
}

impl TestRunner {
    pub fn new() -> Self {
        // Arrange simulator
        let mut ledger = LedgerSimulatorBuilder::new().build();

        // Arrange accounts
        let owner_account = SimAccount::new(&mut ledger);
        let alice_account = SimAccount::new(&mut ledger);
        let bob_account = SimAccount::new(&mut ledger);

        // Create platform helper
        let platform = HelperPlatform::new(&mut ledger, owner_account.clone());

        // Return environment
        TestRunner {
            // Simulation
            ledger,
            // Accounts
            owner_account,
            alice_account,
            bob_account,
            // Components
            platform,
            // Integrations
            weftv2: None,
        }
    }

    pub fn new_weft() -> Self {
        let mut base = TestRunner::new();
        let weftv2 = HelperWeftV2::new(&mut base);

        TestRunner { weftv2: Some(weftv2), ..base }
    }
}
