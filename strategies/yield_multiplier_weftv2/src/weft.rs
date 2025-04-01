use scrypto::prelude::*;

/* --------------- CDP Breakdown -------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct CDPHealthChecker {
    pub total_loan_value: Decimal,
    pub total_adjusted_loan_value: Decimal,

    pub total_collateral_value: Decimal,
    pub total_health_collateral_value: Decimal,
    pub total_liquidation_collateral_value: Decimal,

    pub collateral_isolation_groups: IndexSet<u16>,
    pub loan_excluded_isolation_groups: IndexSet<u16>,

    pub health_ltv: Decimal,
    pub liquidation_ltv: Decimal,

    pub discounted_nft_collateral_value: Decimal,
    pub discounted_collateral_value: Decimal,

    pub loan_positions: IndexMap<ResourceAddress, LoanPositionData>,
    pub collateral_positions: IndexMap<ResourceAddress, CollateralPositionData>,
    pub nft_collateral_positions: IndexMap<ResourceAddress, IndexMap<NonFungibleLocalId, NFTCollateralPositionData>>,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct LoanPositionData {
    pub price: Decimal,
    pub units: Decimal,
    pub amount: Decimal,
    pub value: Decimal,
    pub adjusted_value: Decimal,
    pub config: LoanConfig,
    pub config_version: u64,
    pub resource_config: LoanResourceConfig,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct LoanConfig {
    /// Config description
    pub description: String,

    /// Define a an inflation factor on a loan asset to help mitigate potential risk in borrowing this asset
    pub loan_value_factor: Decimal,

    /// Define the maximum share of a loan that could be repay in a fungible collateral liquidation
    pub loan_close_factor: Decimal,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct LoanResourceConfig {
    pub loan_config_id: u16,
    pub excluded_isolation_group_ids: IndexSet<u16>,
    pub efficiency_group_id: Option<u16>,
}
#[derive(ScryptoSbor, Debug, Clone)]
pub struct CollateralPositionData {
    pub price: Decimal,
    pub amount: Decimal,
    pub value: Decimal,
    pub health_value: Decimal,
    pub liquidation_value: Decimal,
    pub discounted_value: Decimal,
    pub config: CollateralConfig,
    pub config_version: CollateralConfigVersion,
    pub resource_config: CollateralResourceConfig,
    pub is_from_nft: bool,
    pub resource_type: RegisteredResourceType,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CollateralConfig {
    /// Config description
    pub description: String,

    pub loan_to_value_ratio: Decimal,

    pub liquidation_threshold_spread: Decimal,

    pub liquidation_bonus_rate: Decimal,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CollateralResourceConfig {
    pub collateral_config_id: u16,
    pub isolation_group_id: Option<u16>,
    pub efficiency_group_ids: IndexSet<u16>,
}

#[derive(ScryptoSbor, Debug, Clone, Copy, PartialEq)]
pub enum RegisteredResourceType {
    RegisteredToken,
    LSU(ComponentAddress),
    DepositUnit(ResourceAddress),
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct NFTCollateralPositionData {
    pub underlying_positions: IndexMap<ResourceAddress, CollateralPositionData>,
    pub value: NFTLiquidationValue,
    pub max_allowed_discounted_value: Decimal,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct NFTLiquidationValue {
    pub value: Decimal,
    pub discounted_value: Decimal,
    pub loan_payment_value: Decimal,
    pub compensation_value: Decimal,
    pub liquidation_fee: Decimal,
    pub resource_type: RegisteredNFTResourceType,
}

#[derive(ScryptoSbor, Debug, Clone, Copy, Default)]
pub enum RegisteredNFTResourceType {
    #[default]
    RegisteredNFT,
    ClaimNFT(ComponentAddress),
}

/* --------------- Raw CDP Data --------------- */
/// Struct definition to store CDP data.
#[derive(ScryptoSbor, NonFungibleData, Debug, Clone)]
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

#[derive(ScryptoSbor, Debug, Clone, PartialEq, Copy)]
pub enum EfficiencyMode {
    None,
    EfficiencyGroup(u16),
    IdenticalResource,
}

#[derive(ScryptoSbor, Debug, Clone, PartialEq, Copy)]
pub struct CollateralConfigVersion {
    pub entry_version: u64,
    pub efficiency_mode: EfficiencyMode,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct CollateralInfo {
    pub amount: Decimal,
    pub config_version: CollateralConfigVersion,
}

#[derive(ScryptoSbor, Debug, Clone, Default)]
pub struct NFTCollateralInfo {
    pub nft_ids: IndexSet<NonFungibleLocalId>,
    pub config_version: IndexMap<ResourceAddress, CollateralConfigVersion>,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct LoanInfo {
    pub units: Decimal,
    pub config_version: u64,
}
