/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::ServiceValue;

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum ClusterService {
    OpenAccount,
    CloseAccount,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterServiceManager {
    pub can_open_account: ServiceValue,
    pub can_close_account: ServiceValue,
}

impl ClusterServiceManager {
    pub fn new() -> Self {
        Self { can_open_account: ServiceValue::yes(), can_close_account: ServiceValue::yes() }
    }

    pub fn update_service(&mut self, service: ClusterService, value: bool, locked: bool) {
        let set = ServiceValue { value, locked };

        match service {
            ClusterService::OpenAccount => self.can_open_account = set,
            ClusterService::CloseAccount => self.can_close_account = set,
        };
    }

    pub fn get_service(&self, service: ClusterService) -> ServiceValue {
        match service {
            ClusterService::OpenAccount => self.can_open_account,
            ClusterService::CloseAccount => self.can_close_account,
        }
    }
}
