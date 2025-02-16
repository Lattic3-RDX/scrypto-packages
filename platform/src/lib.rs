use scrypto::prelude::*;

#[blueprint]
mod platform {
    struct Platform {
        // Authorisation
        component_address: ComponentAddress,
        // Links
        link_badge_manager: ResourceManager,
        linked_clusters: KeyValueStore<PackageAddress, Vec<ComponentAddress>>,
        linked_count: u64,
    }

    impl Platform {
        //] --------------- Instantiation -------------- */
        // Instantiate with custom rules and badges
        pub fn instantiate_custom() -> () {}

        // Instantiate with an existing owner badge; mint new badges
        pub fn instantiate_with_owner() -> () {}

        // Instantiate and mint a new owner/admin/etc. badge
        pub fn instantiate() -> () {}

        //] -------------- Cluster Linking ------------- */
        // Link a cluster by calling `fn receive_link_badge()` on it
        pub fn link_cluster() -> () {}

        pub fn unlink_cluster() -> () {}
    }
}
