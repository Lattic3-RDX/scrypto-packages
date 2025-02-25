use scrypto_test::prelude::*;

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

pub fn register_cdp() {
    let manifest_builder = ManifestBuilder::new().lock_fee_from_faucet().create_non_fungible_resource(
        OwnerRole::None,
        NonFungibleIdType::Integer,
        true,
        NonFungibleResourceRoles::default(),
        metadata!(
            init {
                "name" => "Example NF", locked;
            }
        ),
        None::<IndexMap<NonFungibleLocalId, CDPData>>,
    );
}
