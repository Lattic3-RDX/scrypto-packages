use crate::helpers::prelude::*;
use scrypto_test::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct Asset {
    pub address: ResourceAddress,
    pub price: Decimal,
}

#[derive(Debug, Clone, Copy)]
pub struct Faucet {
    pub usdt: Asset,
    pub usdc: Asset,
    pub xwbtc: Asset,
    pub hug: Asset,
}

impl Faucet {
    pub fn new(ledger: &mut Ledger, owner: SimAccount) -> Self {
        let usdt = Self::new_asset(ledger, owner, "USDT", dec!(1));
        let usdc = Self::new_asset(ledger, owner, "USDC", dec!(1));
        let xwbtc = Self::new_asset(ledger, owner, "xwBTC", dec!(88950.86));
        let hug = Self::new_asset(ledger, owner, "HUG", dec!(0.00001846));

        Self { usdt, usdc, xwbtc, hug }
    }

    pub fn new_asset(ledger: &mut Ledger, owner: SimAccount, name: &str, price: Decimal) -> Asset {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_fungible_resource(
                OwnerRole::None,
                true,
                DIVISIBILITY_MAXIMUM,
                FungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter         => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner         => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..FungibleResourceRoles::default()
                },
                metadata! {init {
                    "name" => name, locked;
                    "symbol" => name, locked;
                    "description" => format!("Resource {}", name), locked;
                }},
                None,
            )
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![owner.global_id()]);

        let address = receipt.expect_commit(true).new_resource_addresses()[0];
        Asset { address, price }
    }

    pub fn mint(&mut self, ledger: &mut Ledger, address: ResourceAddress, target: SimAccount, amount: Decimal) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_fungible(address, amount)
            .deposit_entire_worktop(target.address)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![target.global_id()]);
        receipt.expect_commit_success();
    }

    pub fn burn(&mut self, ledger: &mut Ledger, address: ResourceAddress, target: SimAccount, amount: Decimal) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .burn_in_account(target.address, address, amount)
            .build();
        let receipt = ledger.execute_manifest(manifest, vec![target.global_id()]);
        receipt.expect_commit_success();
    }
}
