use crate::helpers::{prelude::*, runner::TestRunner};
use scrypto::prelude::indexmap::IndexMap;
use scrypto_test::prelude::*;

/* ------------------ Helper ------------------ */
pub struct HelperWeftV2 {
    pub cdp: ResourceAddress,
    pub cdp_count: u64,
}

impl HelperWeftV2 {
    pub fn new(runner: &mut TestRunner) -> Self {
        // Create CDP NFT
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_non_fungible_resource(
                OwnerRole::None,
                NonFungibleIdType::Integer,
                true,
                // Allow anyone to mint/burn, keep rest as default
                NonFungibleResourceRoles {
                    mint_roles: Some(MintRoles { minter: Some(AccessRule::AllowAll), minter_updater: Some(AccessRule::AllowAll) }),
                    burn_roles: Some(BurnRoles { burner: Some(AccessRule::AllowAll), burner_updater: Some(AccessRule::AllowAll) }),
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

    pub fn mint(&mut self, runner: &mut TestRunner, target: SimAccount) -> NonFungibleLocalId {
        // Create CDP with mock data
        let data = CDPData {
            minted_at: Instant::new(0i64),
            updated_at: Instant::new(0i64),
            key_image_url: String::new(),
            name: format!("Mock CDP {}", self.cdp_count),
            description: String::new(),
            loans: IndexMap::new(),
            collaterals: IndexMap::new(),
            nft_collaterals: IndexMap::new(),
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
