use scrypto::prelude::*;

#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub struct ServiceValue {
    pub value: bool,
    pub locked: bool,
}

impl ServiceValue {
    pub fn yes() -> Self {
        Self { value: true, locked: false }
    }

    pub fn no() -> Self {
        Self { value: false, locked: false }
    }
}
