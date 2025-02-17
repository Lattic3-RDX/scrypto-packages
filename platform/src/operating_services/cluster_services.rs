/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::operating_services::{OperatingServiceManager, ServiceValue};

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone)]
pub enum ClusterService {
    UpdateBadge,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterServiceManager {
    pub can_update_badge: ServiceValue,
}

impl OperatingServiceManager for ClusterServiceManager {
    fn new() -> Self {
        Self { can_update_badge: ServiceValue { value: true } }
    }

    fn update_service(&mut self, service: ClusterService, value: bool) -> Result<(), String> {
        match service {
            ClusterService::UpdateBadge => self.can_update_badge.value = value,
        };

        Ok(())
    }

    fn get_service(&self, service: ClusterService) -> ServiceValue {
        match service {
            ClusterService::UpdateBadge => self.can_update_badge,
        }
    }
}
