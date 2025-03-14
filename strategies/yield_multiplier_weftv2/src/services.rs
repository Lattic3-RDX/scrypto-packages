/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::{ServiceValue, SetLock};

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum ClusterService {
    OpenAccount,
    CloseAccount,
    Execute,
    Link,
    // Unlink,
    CallLinked,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterServiceManager {
    open_account: ServiceValue,
    close_account: ServiceValue,
    execute: ServiceValue,
    link: ServiceValue,
    // unlink: ServiceValue,
    call_linked: ServiceValue,
}

impl ClusterServiceManager {
    pub fn new() -> Self {
        Self {
            open_account: ServiceValue::yes(),
            close_account: ServiceValue::yes(),
            execute: ServiceValue::yes(),
            link: ServiceValue::yes(),
            // unlink: ServiceValue::yes(),
            call_linked: ServiceValue::yes(),
        }
    }

    pub fn update(&mut self, service: ClusterService, value: bool, lock: SetLock) {
        match service {
            ClusterService::OpenAccount => self.open_account.set(value, lock),
            ClusterService::CloseAccount => self.close_account.set(value, lock),
            ClusterService::Execute => self.execute.set(value, lock),
            ClusterService::Link => self.link.set(value, lock),
            // ClusterService::Unlink => self.unlink.set(value, lock),
            ClusterService::CallLinked => self.call_linked.set(value, lock),
        };
    }

    pub fn get(&self, service: ClusterService) -> bool {
        match service {
            ClusterService::OpenAccount => self.open_account.value,
            ClusterService::CloseAccount => self.close_account.value,
            ClusterService::Execute => self.execute.value,
            ClusterService::Link => self.link.value,
            // ClusterService::Unlink => self.unlink.value,
            ClusterService::CallLinked => self.call_linked.value,
        }
    }
}
