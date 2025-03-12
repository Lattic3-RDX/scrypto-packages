/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::ServiceValue;

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum ClusterService {
    UpdateBadge,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterServiceManager {
    pub can_update_badge: ServiceValue,
}

impl ClusterServiceManager {
    pub fn new() -> Self {
        Self { can_update_badge: ServiceValue::yes() }
    }

    pub fn update_service(&mut self, service: ClusterService, value: bool, locked: bool) {
        let set = ServiceValue { value, locked };

        match service {
            ClusterService::UpdateBadge => self.can_update_badge = set,
        };
    }

    pub fn get_service(&self, service: ClusterService) -> ServiceValue {
        match service {
            ClusterService::UpdateBadge => self.can_update_badge,
        }
    }
}
