/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::ServiceValue;

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

    pub fn update(&mut self, service: ClusterService, value: bool, locked: bool) {
        let set = ServiceValue { value, locked };

        match service {
            ClusterService::OpenAccount => self.open_account = set,
            ClusterService::CloseAccount => self.close_account = set,
            ClusterService::Execute => self.execute = set,
            ClusterService::Link => self.link = set,
            // ClusterService::Unlink => self.unlink = set,
            ClusterService::CallLinked => self.call_linked = set,
        };
    }

    pub fn get(&self, service: ClusterService) -> ServiceValue {
        match service {
            ClusterService::OpenAccount => self.open_account,
            ClusterService::CloseAccount => self.close_account,
            ClusterService::Execute => self.execute,
            ClusterService::Link => self.link,
            // ClusterService::Unlink => self.unlink,
            ClusterService::CallLinked => self.call_linked,
        }
    }
}
