/* ------------------ Imports ----------------- */
use crate::services::cluster_services::ClusterServiceManager;
use scrypto::prelude::*;

/* -------------- Cluster Wrapper ------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterWrapper {
    pub cluster_address: ComponentAddress,
    pub blueprint_id: BlueprintId,
    pub link_id: NonFungibleLocalId,
    pub services: ClusterServiceManager,
}

impl ClusterWrapper {
    pub fn new(cluster_address: ComponentAddress, blueprint_id: BlueprintId, link_id: NonFungibleLocalId) -> Self {
        Self { cluster_address, blueprint_id, link_id, services: ClusterServiceManager::new() }
    }

    pub fn call<T: ScryptoDecode>(&self, method_name: &str, args: Vec<u8>) -> T {
        let cluster: Global<AnyComponent> = self.cluster_address.into();
        cluster.call_raw::<T>(method_name, args)
    }
}
