use scrypto::prelude::*;

#[derive(NonFungibleData, ScryptoSbor, Debug, Clone)]
pub struct ExecutionTerms {}

impl ExecutionTerms {
    pub fn new() -> Self {
        ExecutionTerms {}
    }
}
