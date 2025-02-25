use crate::helpers::{platform::HelperPlatform, prelude::*};
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
        TestRunner { ledger, owner_account, alice_account, bob_account, platform }
    }
}
