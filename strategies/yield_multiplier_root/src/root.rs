use scrypto::prelude::*;

/* ------------- Price Oracle Data ------------ */
#[derive(ScryptoSbor, Clone)]
pub struct PriceInfo {
    pub timestamp: i64,
    pub price: Decimal,
}
/* --------------- CDP Breakdown -------------- */
#[derive(ScryptoSbor, NonFungibleData, Clone, Debug)]
pub struct CollaterizedDebtPositionData {
    /// Image to display when exploring Radix transactions
    #[mutable]
    pub key_image_url: String,

    /// Name of the CDP
    #[mutable]
    pub name: String,

    /// Textual description of the CDP
    #[mutable]
    pub description: String,

    /// Immutable timestamp of CDP minting
    pub minted_at: i64,

    /// Timestamp of CDP update
    #[mutable]
    pub updated_at: i64,

    /// Type of the CDP
    #[mutable]
    pub cdp_type: CDPType,

    /// Map of collateral values, having the asset as key and the unit amount as value.
    /// Here, `PreciseDecimal` helps in keeping precision in computations
    /// even if the actual amount will require to be expressed as `Decimal`
    #[mutable]
    pub collaterals: IndexMap<ResourceAddress, PreciseDecimal>,

    /// Map of loaned values, having the asset as key and the unit amount as value.
    /// Here, `PreciseDecimal` helps in keeping precision in computation
    /// even if the actual amount will require to be expressed as `Decimal`
    #[mutable]
    pub loans: IndexMap<ResourceAddress, PreciseDecimal>,

    /// The maximum amount of liquidable value for this collateralized debt position
    #[mutable]
    pub liquidable: Option<Decimal>,
}

#[derive(ScryptoSbor, Clone, PartialEq, Debug)]
pub enum CDPType {
    /// A CDP where an user is directly responsible of his borrows and collaterals, subject to liquidation
    Standard,
}
