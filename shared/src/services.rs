use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub struct ServiceValue {
    pub value: bool,
}
