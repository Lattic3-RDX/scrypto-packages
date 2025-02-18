/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::ServiceValue;

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum PlatformService {
    MintBadge,
    UpdateBadge,
    AuthoriseTx,
    LinkCluster,
    UnlinkCluster,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct PlatformServiceManager {
    pub can_mint_badge: ServiceValue,
    pub can_update_badge: ServiceValue,
    pub can_authorise_tx: ServiceValue,
    pub can_link_cluster: ServiceValue,
    pub can_unlink_cluster: ServiceValue,
}

impl PlatformServiceManager {
    pub fn new() -> Self {
        Self {
            can_mint_badge: ServiceValue { value: true },
            can_update_badge: ServiceValue { value: true },
            can_authorise_tx: ServiceValue { value: true },
            can_link_cluster: ServiceValue { value: true },
            can_unlink_cluster: ServiceValue { value: true },
        }
    }

    pub fn update(&mut self, service: PlatformService, value: bool) -> Result<(), String> {
        match service {
            PlatformService::MintBadge => self.can_mint_badge.value = value,
            PlatformService::UpdateBadge => self.can_update_badge.value = value,
            PlatformService::AuthoriseTx => self.can_authorise_tx.value = value,
            PlatformService::LinkCluster => self.can_link_cluster.value = value,
            PlatformService::UnlinkCluster => self.can_unlink_cluster.value = value,
        };

        Ok(())
    }

    pub fn get(&self, service: PlatformService) -> ServiceValue {
        match service {
            PlatformService::MintBadge => self.can_mint_badge,
            PlatformService::UpdateBadge => self.can_update_badge,
            PlatformService::AuthoriseTx => self.can_authorise_tx,
            PlatformService::LinkCluster => self.can_link_cluster,
            PlatformService::UnlinkCluster => self.can_unlink_cluster,
        }
    }
}
