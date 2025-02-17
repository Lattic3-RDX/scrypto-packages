use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor)]
pub struct Link {
    pub issuing_platform: ComponentAddress,
    pub linked_cluster: ComponentAddress,
}

impl Link {
    pub fn new(issuing_platform: ComponentAddress, linked_cluster: ComponentAddress) -> Self {
        Self { issuing_platform, linked_cluster }
    }
}
