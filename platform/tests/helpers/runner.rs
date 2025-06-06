use crate::helpers::{faucet::Faucet, platform::PlatformFactory, prelude::*};
use scrypto_test::{prelude::*, utils::dump_manifest_to_file_system};
use std::{fs::create_dir_all, path::PathBuf};

use super::platform::Platform;

pub struct Runner {
    // Simulation
    pub ledger: Ledger,
    // Accounts
    pub owner_account: SimAccount,
    // pub owner_badge: ResourceAddress,
    pub alice_account: SimAccount,
    pub bob_account: SimAccount,
    // Components
    pub faucet: Faucet,
    pub platform_factory: PlatformFactory,
}

impl Runner {
    pub fn new() -> Self {
        // Arrange simulator
        let mut ledger = LedgerSimulatorBuilder::new().build();

        // Arrange accounts
        let owner_account = SimAccount::new(&mut ledger);
        let alice_account = SimAccount::new(&mut ledger);
        let bob_account = SimAccount::new(&mut ledger);

        // Create faucet
        let faucet = Faucet::new(&mut ledger, owner_account.clone());

        // Create platform factory
        let platform_factory = PlatformFactory::new(&mut ledger);

        // Return environment
        Runner {
            // Simulation
            ledger,
            // Accounts
            owner_account,
            // owner_badge: platform.owner_badge,
            alice_account,
            bob_account,
            // Components
            faucet,
            platform_factory,
        }
    }

    pub fn new_base() -> (Self, Platform) {
        let mut runner = Runner::new();

        let platform = runner.platform_factory.instantiate(&mut runner.ledger, runner.owner_account);

        (runner, platform)
    }

    pub fn exec(&mut self, name: &str, manifest_builder: ManifestBuilder, account: &SimAccount, path: Option<&str>) -> TransactionReceipt {
        // Build manifest
        let manifest = manifest_builder.build();

        // Execute and record manifest output
        let receipt = self.ledger.execute_manifest(manifest, vec![account.global_id()]);

        let path = PathBuf::from(
            format!(
                "./output{}",
                match path {
                    Some(path) => format!("/{}", path),
                    None => "".to_string(),
                }
            )
            .as_str(),
        );

        assert!(!path.is_file(), "Output path is a file");
        create_dir_all(&path).unwrap();

        let text = format!("{:?}", receipt);
        std::fs::write(path.join(format!("{}.txt", name)), text).unwrap();

        receipt
    }

    pub fn exec_and_dump(&mut self, name: &str, manifest_builder: ManifestBuilder, account: &SimAccount, path: Option<&str>) -> TransactionReceipt {
        // Build and dump manifest
        let manifest = manifest_builder.build();

        dump_manifest_to_file_system(
            &manifest,
            format!(
                "./manifests{}",
                match path {
                    Some(path) => format!("/{}", path),
                    None => "".to_string(),
                }
            ),
            Some(name),
            &NetworkDefinition::simulator(),
        )
        .err();

        // Execute and record manifest output
        let receipt = self.ledger.execute_manifest(manifest, vec![account.global_id()]);

        let path = PathBuf::from(
            format!(
                "./output{}",
                match path {
                    Some(path) => format!("/{}", path),
                    None => "".to_string(),
                }
            )
            .as_str(),
        );

        assert!(!path.is_file(), "Output path is a file");
        create_dir_all(&path).unwrap();

        let text = format!("{:?}", receipt);
        std::fs::write(path.join(format!("{}.txt", name)), text).unwrap();

        receipt
    }
}
