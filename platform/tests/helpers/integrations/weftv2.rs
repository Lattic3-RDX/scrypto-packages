use crate::helpers::prelude::*;
use scrypto::prelude::indexmap::{IndexMap, IndexSet};
use scrypto_test::prelude::*;

//] ------------ Mock Implementation ----------- */
#[derive(Debug, Clone, Copy)]
pub struct MockWeftV2 {
    pub cdp: ResourceAddress,
    pub cdp_count: u64,
}

impl MockWeftV2 {
    pub fn new(runner: &mut Runner) -> Self {
        // Create CDP NFT
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                // Allow anyone to mint/burn, keep rest as default
                NonFungibleResourceRoles {
                    mint_roles: mint_roles! {
                        minter         => rule!(allow_all);
                        minter_updater => rule!(deny_all);
                    },
                    burn_roles: burn_roles! {
                        burner         => rule!(allow_all);
                        burner_updater => rule!(deny_all);
                    },
                    ..NonFungibleResourceRoles::default()
                },
                metadata!(
                    init {
                        "name" => "Mock WeftV2 CDP", locked;
                    }
                ),
                None::<IndexMap<NonFungibleLocalId, CDPData>>,
            )
            .build();

        // Execute manifest
        let receipt = runner
            .ledger
            .execute_manifest(manifest, vec![NonFungibleGlobalId::from_public_key(&runner.owner_account.public_key)]);

        // Collect output
        let cdp = receipt.expect_commit_success().new_resource_addresses()[0];

        Self { cdp, cdp_count: 0 }
    }

    pub fn mint(
        &mut self,
        runner: &mut Runner,
        target: SimAccount,
        loans: Option<IndexMap<ResourceAddress, Decimal>>,
        collaterals: Option<IndexMap<ResourceAddress, Decimal>>,
        nft: bool,
    ) -> NonFungibleLocalId {
        // Convert loan mapping with decimal input to one with loan info
        let loans: IndexMap<ResourceAddress, LoanInfo> = match loans {
            Some(loans) => loans
                .iter()
                .map(|(&address, &units)| (address, LoanInfo { units, config_version: 1 }))
                .collect(),
            None => IndexMap::new(),
        };

        // Convert collateral mapping with decimal input to one with collateral info
        let collaterals: IndexMap<ResourceAddress, CollateralInfo> = match collaterals {
            Some(collaterals) => collaterals
                .iter()
                .map(|(&address, &amount)| {
                    let info = CollateralInfo {
                        amount,
                        config_version: CollateralConfigVersion { entry_version: 1, efficiency_mode: EfficiencyMode::None },
                    };

                    (address, info)
                })
                .collect(),
            None => IndexMap::new(),
        };

        // Create mock NFT collateral mapping
        let nft_collaterals: IndexMap<ResourceAddress, NFTCollateralInfo> = match nft {
            true => {
                let info = NFTCollateralInfo { nft_ids: IndexSet::new(), config_version: IndexMap::new() };

                indexmap! { self.cdp => info }
            }
            false => IndexMap::new(),
        };

        // Create CDP with mock data
        let data = CDPData {
            minted_at: Instant::new(0i64),
            updated_at: Instant::new(0i64),
            key_image_url: String::new(),
            name: format!("Mock CDP {}", self.cdp_count),
            description: String::new(),
            loans,
            collaterals,
            nft_collaterals,
        };

        // Mint the CDP
        let local_id = NonFungibleLocalId::Integer(self.cdp_count.into());
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .mint_non_fungible(self.cdp, [(local_id.clone(), data)])
            .deposit_entire_worktop(target.address)
            .build();

        // Increment the CDP count
        self.cdp_count += 1;

        // Execute the manifest
        let receipt = runner.ledger.execute_manifest(manifest, vec![target.global_id()]);
        receipt.expect_commit_success();

        // Return the local id
        local_id
    }

    pub fn mint_empty(&mut self, runner: &mut Runner, target: SimAccount) -> NonFungibleLocalId {
        self.mint(runner, target, None, None, false)
    }
}

/* ---------------- Integration --------------- */
#[derive(ScryptoSbor, Debug, Clone, PartialEq, Copy, ManifestSbor)]
pub enum EfficiencyMode {
    None,
    EfficiencyGroup(u16),
    IdenticalResource,
}

#[derive(ScryptoSbor, Debug, Clone, PartialEq, Copy, ManifestSbor)]
pub struct CollateralConfigVersion {
    pub entry_version: u64,
    pub efficiency_mode: EfficiencyMode,
}

#[derive(ScryptoSbor, Debug, Clone, ManifestSbor)]
pub struct CollateralInfo {
    pub amount: Decimal,
    pub config_version: CollateralConfigVersion,
}

#[derive(ScryptoSbor, Debug, Clone, Default, ManifestSbor)]
pub struct NFTCollateralInfo {
    pub nft_ids: IndexSet<NonFungibleLocalId>,
    pub config_version: IndexMap<ResourceAddress, CollateralConfigVersion>,
}

#[derive(ScryptoSbor, Debug, Clone, ManifestSbor)]
pub struct LoanInfo {
    pub units: Decimal,
    pub config_version: u64,
}

/// Struct definition to store CDP data.
#[derive(ScryptoSbor, NonFungibleData, Debug, Clone, ManifestSbor)]
pub struct CDPData {
    // #[immutable]
    minted_at: Instant,
    #[mutable]
    updated_at: Instant,

    // Wallet metadata
    #[mutable]
    key_image_url: String,
    #[mutable]
    name: String,
    #[mutable]
    description: String,

    // Positions data
    #[mutable]
    pub loans: IndexMap<ResourceAddress, LoanInfo>,
    #[mutable]
    pub collaterals: IndexMap<ResourceAddress, CollateralInfo>,
    #[mutable]
    pub nft_collaterals: IndexMap<ResourceAddress, NFTCollateralInfo>,
}
