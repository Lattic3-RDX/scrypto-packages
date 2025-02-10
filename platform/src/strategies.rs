/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* ------------------- Badge ------------------ */
/// Represents a strategy that has been verified by the central platform.
///
/// This struct is used to store information about a verified strategy,
/// including its name and the component address of the entity that verified it.
/// None of the fields are mutable, to prevent misrepresentation of the verified status.
#[derive(NonFungibleData, ScryptoSbor, Debug)]
pub struct VerifiedStrategy {
    pub name: String,
    pub verified_by: ComponentAddress,
}
