/* ------------------ Imports ----------------- */
use scrypto::prelude::*;
use shared::services::ServiceValue;

/* ------------ Operating Services ------------ */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
pub enum PlatformService {
    MintBadge,
    OpenAccount,
    CloseAccount,
    // AuthoriseExecution,
    LinkCluster,
    UnlinkCluster,
}

#[derive(ScryptoSbor, Debug, Clone)]
pub struct PlatformServiceManager {
    pub can_mint_badge: ServiceValue,
    pub can_open_badge: ServiceValue,
    pub can_close_badge: ServiceValue,
    // pub can_authorise_execution: ServiceValue,
    pub can_link_cluster: ServiceValue,
    pub can_unlink_cluster: ServiceValue,
}

impl PlatformServiceManager {
    pub fn new() -> Self {
        Self {
            can_mint_badge: ServiceValue::yes(),
            can_open_badge: ServiceValue::yes(),
            can_close_badge: ServiceValue::yes(),
            // can_authorise_execution: ServiceValue::yes(),
            can_link_cluster: ServiceValue::yes(),
            can_unlink_cluster: ServiceValue::yes(),
        }
    }

    pub fn update(&mut self, service: PlatformService, value: bool, locked: bool) {
        let set = ServiceValue { value, locked };

        match service {
            PlatformService::MintBadge => self.can_mint_badge = set,
            PlatformService::OpenAccount => self.can_open_badge = set,
            PlatformService::CloseAccount => self.can_close_badge = set,
            // PlatformService::AuthoriseExecution => self.can_authorise_execution = set,
            PlatformService::LinkCluster => self.can_link_cluster = set,
            PlatformService::UnlinkCluster => self.can_unlink_cluster = set,
        };
    }

    pub fn get(&self, service: PlatformService) -> ServiceValue {
        match service {
            PlatformService::MintBadge => self.can_mint_badge,
            PlatformService::OpenAccount => self.can_open_badge,
            PlatformService::CloseAccount => self.can_close_badge,
            // PlatformService::AuthoriseExecution => self.can_authorise_execution,
            PlatformService::LinkCluster => self.can_link_cluster,
            PlatformService::UnlinkCluster => self.can_unlink_cluster,
        }
    }
}
