/* ------------------ Imports ----------------- */
use crate::operating_services::cluster_services::ClusterServiceManager;
use scrypto::prelude::*;

/* -------------- Cluster Wrapper ------------- */
#[derive(ScryptoSbor, Debug, Clone)]
pub struct ClusterWrapper {
    pub cluster_address: ComponentAddress,
    pub package_address: PackageAddress,
    pub link_id: Option<NonFungibleLocalId>,
    pub services: ClusterServiceManager,
}

impl ClusterWrapper {
    pub fn new(cluster_address: ComponentAddress, package_address: PackageAddress, link_id: Option<NonFungibleLocalId>) -> Self {
        Self {
            cluster_address,
            package_address,
            link_id,
            services: ClusterServiceManager { can_update_badge: false },
        }
    }

    pub fn call<T: ScryptoDecode>(&self, method_name: &str, args: Vec<u8>) -> T {
        let cluster: Global<AnyComponent> = self.cluster_address.into();
        cluster.call_raw::<T>(method_name, args)
    }
}
