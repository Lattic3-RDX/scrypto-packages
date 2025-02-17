/* ------------------ Imports ----------------- */
use crate::clusters::ClusterWrapper;
use crate::services::cluster_services::ClusterService;
use scrypto::prelude::*;
use shared::links::Link;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod platform {
    struct Platform {
        // Authorisation
        component_address: ComponentAddress,
        // Links
        link_badge_manager: NonFungibleResourceManager,
        linked_clusters: KeyValueStore<ComponentAddress, ClusterWrapper>,
        linked_count: u64,
        unused_linked_ids: IndexSet<u64>,
    }

    impl Platform {
        //] --------------- Instantiation -------------- */
        // Instantiate with custom rules and badges
        pub fn instantiate_custom() -> () {}

        // Instantiate with an existing owner badge; mint new badges
        pub fn instantiate_with_owner() -> () {}

        // Instantiate and mint a new owner/admin/etc. badge
        pub fn instantiate() -> (Global<Platform>, FungibleBucket) {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(Self::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            let owner_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {init {
                    "name"        => "Lattic3 Owner Badge", locked;
                    "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
                }})
                .mint_initial_supply(1);
            let owner_access_rule: AccessRule = rule!(require(owner_badge.resource_address()));
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_access_rule.clone());

            //] Links
            let link_badge_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<Link>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//Link", locked;
                    "description"     => "Badge linking this cluster to the Lattic3 platform.", locked;
                    // "dapp_definition" => dapp_definition_address, updatable;
                }})
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater         => component_access_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .mint_roles(mint_roles! {
                    minter         => component_access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner         => component_access_rule.clone();
                    burner_updater => rule!(deny_all);
                })
                .recall_roles(recall_roles! {
                    recaller         => rule!(require(owner_badge.resource_address()) || require(global_caller(component_address)));
                    recaller_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            //] Component Instantisation
            // Metadata
            let component_metadata = metadata! {
                roles {
                    metadata_setter         => OWNER;
                    metadata_setter_updater => OWNER;
                    metadata_locker         => OWNER;
                    metadata_locker_updater => rule!(deny_all);
                },
                init {
                    "name"            => "L3//Platform", updatable;
                    "description"     => "Platform component for the Lattic3 DeFi strategy aggregator.", updatable;
                    // "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            // let component_roles = roles! {
            //     platform => platform_access_rule.clone();
            // };

            // Instantisation
            let initial_state = Self {
                // Authorisation
                component_address,
                // Links
                link_badge_manager,
                linked_clusters: KeyValueStore::new(),
                linked_count: 0,
                unused_linked_ids: IndexSet::new(),
            };

            let component: Global<Self> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                // .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            (component, owner_badge)
        }

        //] ------------------- Links ------------------ */
        // RESTRICT can_link/component
        pub fn link_cluster(&mut self, cluster_address: ComponentAddress, package_address: PackageAddress) -> Result<(), String> {
            // Ensure that the cluster hasn't already been linked
            if self.linked_clusters.get(&cluster_address).is_some() {
                return Err("Cluster already linked".to_string());
            }

            // Create link badge
            let link_data = Link::new(self.component_address, cluster_address);
            let link_id = NonFungibleLocalId::Integer(self.linked_count.into());
            let link_badge = self.link_badge_manager.mint_non_fungible(&link_id, link_data);

            self.linked_count += 1;

            // Create ClusterWrapper
            let wrapper = ClusterWrapper::new(cluster_address, package_address, Some(link_id));

            // Deposit badge into cluster and insert into KV
            wrapper.call::<Result<(), String>>("handle_link", scrypto_args!(link_badge))?;
            self.linked_clusters.insert(cluster_address, wrapper);

            Ok(())
        }

        // RESTRICT can_link/component
        pub fn unlink_cluster(&mut self, cluster_address: ComponentAddress) -> Result<(), String> {
            // Sanity checks
            if self.linked_clusters.get(&cluster_address).is_none() {
                return Err("Cluster already linked".to_string());
            }

            let wrapper = self.linked_clusters.get(&cluster_address).unwrap();

            self.linked_clusters.remove(&cluster_address);

            Ok(())
        }

        pub fn update_cluster_service(&mut self, cluster_address: ComponentAddress, service: ClusterService, value: bool) -> Result<(), String> {
            let mut wrapper = self
                .linked_clusters
                .get_mut(&cluster_address)
                .ok_or("Cluster with given address not linked".to_string())?;
            wrapper.services.update_service(service, value)
        }

        // PUBLIC
        pub fn validate_link_proof(&self) -> bool {
            true
        }
    }
}
