// Libraries
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

    pub fn set(&mut self, value: bool, lock: SetLock) {
        match lock {
            SetLock::None => {
                assert!(!self.locked);
                self.value = value;
            }
            SetLock::Update(lock) => {
                self.value = value;
                self.locked = lock;
            }
        }
    }
}

#[derive(ScryptoSbor, Debug, PartialEq, Clone, Copy)]
pub enum SetLock {
    None,
    Update(bool),
}
