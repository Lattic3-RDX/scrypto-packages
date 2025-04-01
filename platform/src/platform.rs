/* ------------------ Imports ----------------- */
// Modules
use crate::clusters::ClusterWrapper;
use crate::services::cluster_services::{ClusterService, ClusterServiceManager};
use crate::services::platform_services::{PlatformService, PlatformServiceManager};
// Shared Modules
use shared::links::Link;
use shared::services::ServiceValue;
use shared::users::User;
// Libraries
use scrypto::prelude::*;

/* ----------------- Blueprint ---------------- */
type Unit = ();

#[blueprint]
#[types(
    Unit,
    ComponentAddress,
    ResourceAddress,
    NonFungibleResourceManager,
    NonFungibleLocalId,
    Decimal,
    u64,
    Link,
    User,
    Instant,
    ClusterWrapper,
    BlueprintId,
    PlatformServiceManager,
    PlatformService,
    ClusterServiceManager,
    ClusterService,
    ServiceValue
)]
mod platform {

    //] --------------- Scrypto Setup -------------- */
    enable_method_auth! {
        roles {
            can_manage_links    => updatable_by: [OWNER];
            can_update_services => updatable_by: [OWNER];
            can_lock_services   => updatable_by: [OWNER];
        },
        methods {
            // User
            new_user      => PUBLIC;
            open_account  => PUBLIC;
            close_account => PUBLIC;
            // Links
            link_cluster   => restrict_to: [can_manage_links];
            unlink_cluster => restrict_to: [can_manage_links];
            update_cluster_service              => restrict_to: [can_update_services, can_lock_services];
            update_cluster_service_and_set_lock => restrict_to: [can_lock_services];
            // Platform
            get_user_badge_address => PUBLIC;
            get_link_badge_address => PUBLIC;
            new_admin_badge        => restrict_to: [OWNER];
            update_service              => restrict_to: [can_update_services, can_lock_services];
            update_service_and_set_lock => restrict_to: [can_lock_services];
        }
    }

    //] ------------ Platform Blueprint ------------ */
    struct Platform {
        // Authorisation
        component_address: ComponentAddress,
        admin_badge_manager: NonFungibleResourceManager,
        admin_count: u64,
        // User badges
        user_badge_manager: NonFungibleResourceManager,
        user_count: u64,
        // Operating services
        services: PlatformServiceManager,
        // Links
        link_badge_manager: NonFungibleResourceManager,
        linked_clusters: KeyValueStore<ComponentAddress, ClusterWrapper>,
        linked_count: u64,
    }

    impl Platform {
        /// Instantiates a new `Platform` component, with a specified `dapp_definition_address`.
        /// Also creates an owner badge for the platform.
        ///
        /// # Parameters
        /// - `dapp_definition_address`: The address of the dAppDefinition account.
        ///
        /// # Returns
        /// - `Global<Platform>`: A global reference to the instantiated `Platform` component.
        /// - `FungibleBucket`: The owner badge of the `Platform` component.
        pub fn instantiate(dapp_definition_address: ComponentAddress) -> (Global<Platform>, FungibleBucket) {
            // Component owner
            let owner_badge: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {init {
                    "name"        => "Lattic3 Owner", locked;
                    "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
                }})
                .mint_initial_supply(1);
            let owner_rule: AccessRule = rule!(require(owner_badge.resource_address()));

            let platform: Global<Platform> = Self::instantiate_advanced(owner_rule, dapp_definition_address);

            (platform, owner_badge)
        }

        /// Instantiates a new `Platform` component, with a specified `owner_rule` and `dapp_definition_address`.
        /// Creates an admin badge, link badge, and user badge.
        ///
        /// # Parameters
        /// - `owner_rule`: Access rule specifying the owner of the component.
        /// - `dapp_definition_address`: The address of the dAppDefinition account.
        ///
        /// # Returns
        /// - `Global<Platform>`: A global reference to the instantiated `Platform` component.
        pub fn instantiate_advanced(owner_rule: AccessRule, dapp_definition_address: ComponentAddress) -> Global<Platform> {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(Self::blueprint_id());

            //] Authorisation
            // Component
            let component_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_rule.clone());

            // Admin badge
            let admin_badge_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<()>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//Admin", locked;
                    "description"     => "Badge used to denote an admin's ownership over accounts in Lattic3 clusters.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }})
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater         => component_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .mint_roles(mint_roles! {
                    minter         => component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner         => owner_rule.clone();
                    burner_updater => owner_rule.clone();
                })
                .recall_roles(recall_roles! {
                    recaller         => owner_rule.clone();
                    recaller_updater => owner_rule.clone();
                })
                .create_with_no_initial_supply();
            let admin_rule: AccessRule = rule!(require(admin_badge_manager.address()));

            // User badge
            let user_badge_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<User>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//User", locked;
                    "description"     => "Badge used to denote a user's ownership over accounts in Lattic3 clusters.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }})
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater         => component_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .mint_roles(mint_roles! {
                    minter         => component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner         => component_rule.clone();
                    burner_updater => rule!(deny_all);
                })
                .create_with_no_initial_supply();

            // Link badge
            let link_badge_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<Link>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//Link", locked;
                    "description"     => "Badge linking this cluster to the Lattic3 platform.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }})
                .non_fungible_data_update_roles(non_fungible_data_update_roles! {
                    non_fungible_data_updater         => component_rule.clone();
                    non_fungible_data_updater_updater => rule!(deny_all);
                })
                .mint_roles(mint_roles! {
                    minter         => component_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner         => owner_rule.clone();
                    burner_updater => owner_rule.clone();
                })
                .recall_roles(recall_roles! {
                    recaller         => owner_rule.clone();
                    recaller_updater => owner_rule.clone();
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
                    "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            let component_roles = roles! {
                can_manage_links    => OWNER;
                can_update_services => admin_rule;
                can_lock_services   => OWNER;
            };

            // Instantisation
            let initial_state = Self {
                // Authorisation
                component_address,
                admin_badge_manager,
                admin_count: 0,
                // User badges
                user_badge_manager,
                user_count: 0,
                // Operating services
                services: PlatformServiceManager::new(),
                // Links
                link_badge_manager,
                linked_clusters: KeyValueStore::new(),
                linked_count: 0,
            };

            let component: Global<Self> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }

        //] ------------------- User ------------------- */
        /// Mint a new user badge; accounts empty by default.
        ///
        /// # Panics
        /// - If the service `PlatformService::MintBadge` is disabled.
        /// - If the number of user badges exceeds u64::MAX.
        ///
        /// # Returns
        /// - `NonFungibleBucket`: The new user badge.
        pub fn new_user(&mut self) -> NonFungibleBucket {
            // Ensure that a new user badge can be minted
            assert!(self.services.get(PlatformService::MintBadge).value, "PlatformService::MintBadge disabled");
            assert!(self.user_count < u64::MAX, "Cannot mint more user badges; at U64 MAX");

            // Create empty user badge
            let badge_data: User = User::new();
            let badge_id = NonFungibleLocalId::Integer(self.user_count.into());

            // Increment user badge count
            self.user_count += 1;

            self.user_badge_manager.mint_non_fungible(&badge_id, badge_data)
        }

        pub fn open_account(&self, link_badge: NonFungibleProof, user_id: NonFungibleLocalId) {
            assert!(
                self.services.get(PlatformService::OpenAccount).value,
                "PlatformService::OpenAccount disabled"
            );

            // Validate the link
            let wrapper = self.__validate_link(link_badge);
            let can_update_badge = wrapper.services.get_service(ClusterService::OpenAccount).value;
            assert_eq!(can_update_badge, true, "ClusterService::OpenAccount disabled");

            // Open account and update badge
            let mut user: User = self.user_badge_manager.get_non_fungible_data::<User>(&user_id);
            user.add_account(wrapper.cluster_address);

            self.user_badge_manager
                .update_non_fungible_data(&user_id, "accounts_in", user.accounts_in);
            self.user_badge_manager.update_non_fungible_data(&user_id, "open", user.open);
        }

        pub fn close_account(&self, link_badge: NonFungibleProof, user_id: NonFungibleLocalId) {
            assert!(
                self.services.get(PlatformService::CloseAccount).value,
                "PlatformService::CloseAccount disabled"
            );

            // Validate the link
            let wrapper = self.__validate_link(link_badge);
            let can_update_badge = wrapper.services.get_service(ClusterService::CloseAccount).value;
            assert_eq!(can_update_badge, true, "ClusterService::CloseAccount disabled");

            // Close account and update badge
            let mut user: User = self.user_badge_manager.get_non_fungible_data::<User>(&user_id);
            user.remove_account(wrapper.cluster_address);

            self.user_badge_manager
                .update_non_fungible_data(&user_id, "accounts_in", user.accounts_in);
            self.user_badge_manager.update_non_fungible_data(&user_id, "open", user.open);
        }

        //] Private
        fn __validate_user(&self, user_badge: NonFungibleProof) -> CheckedNonFungibleProof {
            let valid_user = user_badge.check_with_message(self.user_badge_manager.address(), "User badge not valid");
            assert_eq!(valid_user.amount(), dec!(1), "Invalid user badge quantity");

            valid_user
        }

        //] ------------------- Links ------------------ */
        pub fn link_cluster(&mut self, cluster_address: ComponentAddress) {
            assert!(
                self.services.get(PlatformService::LinkCluster).value,
                "PlatformService::LinkCluster disabled"
            );

            // Ensure that the cluster hasn't already been linked
            assert!(self.linked_clusters.get(&cluster_address).is_none(), "Cluster already linked");

            // Ensure that a new link badge can be minted
            assert!(self.linked_count < u64::MAX, "Cannot link more clusters; at U64 MAX");

            // Create link badge
            let link_data = Link::new(self.component_address, cluster_address);
            let link_id = NonFungibleLocalId::Integer(self.linked_count.into());
            let link_badge = self.link_badge_manager.mint_non_fungible(&link_id, link_data);

            self.linked_count += 1;

            // Create ClusterWrapper
            let blueprint_id = ScryptoVmV1Api::object_get_blueprint_id(cluster_address.as_node_id()); // Fetch blueprint id to group clusters by strategy
            let wrapper = ClusterWrapper::new(cluster_address, blueprint_id, link_id);

            // Deposit badge into cluster and insert into KV
            wrapper.call::<()>("handle_link", scrypto_args!(link_badge));
            self.linked_clusters.insert(cluster_address, wrapper);
        }

        pub fn unlink_cluster(&mut self, cluster_address: ComponentAddress) {
            assert!(
                self.services.get(PlatformService::UnlinkCluster).value,
                "PlatformService::UnlinkCluster disabled"
            );

            // let _wrapper = self.linked_clusters.get(&cluster_address).ok_or("Cluster already linked".to_string());
            assert!(self.linked_clusters.get(&cluster_address).is_some(), "Cluster not linked");

            self.linked_clusters.remove(&cluster_address);
        }

        //] Services
        /// Updates a cluster service, assuming it is not locked.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        ///
        /// # Panics
        /// - If the service is currently locked.
        pub fn update_cluster_service(&mut self, cluster_address: ComponentAddress, service: ClusterService, value: bool) {
            let mut wrapper = self
                .linked_clusters
                .get_mut(&cluster_address)
                .expect("Cluster with given address not linked");
            wrapper.services.update_service(service, value, false);
        }

        /// Updates a cluster service and sets the lock state.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        /// - `locked`: The value to which the lock status of the service is set to.
        pub fn update_cluster_service_and_set_lock(&mut self, cluster_address: ComponentAddress, service: ClusterService, value: bool, locked: bool) {
            let mut wrapper = self
                .linked_clusters
                .get_mut(&cluster_address)
                .expect("Cluster with given address not linked");
            wrapper.services.update_service(service, value, locked);
        }

        //] Private
        fn __validate_link(&self, link_badge: NonFungibleProof) -> ClusterWrapper {
            // Validate the proof
            let valid_link = link_badge.check_with_message(self.link_badge_manager.address(), "Link badge not valid");
            assert_eq!(valid_link.amount(), dec!(1), "Invalid link badge quantity");

            // Deconstruct the link badge
            let link: Link = valid_link.non_fungible().data();
            let cluster_address = link.linked_cluster;
            let wrapper = self.linked_clusters.get(&cluster_address).expect("Cluster not linked");

            // Validate the link
            assert_eq!(link.issuing_platform, self.component_address, "Link badge not issued by this platform");

            // Return the ClusterWrapper
            wrapper.clone()
        }

        //] ----------------- Platform ----------------- */
        /// Returns the ResourceAddress of the user badge.
        pub fn get_user_badge_address(&self) -> ResourceAddress {
            self.user_badge_manager.address()
        }

        /// Returns the ResourceAddress of the link badge.
        pub fn get_link_badge_address(&self) -> ResourceAddress {
            self.link_badge_manager.address()
        }

        /// Mint a new admin badge.
        ///
        /// # Panics
        /// - If the service `PlatformService::MintBadge` is disabled.
        /// - If the number of admin badges exceeds u64::MAX.
        ///
        /// # Returns
        /// - `NonFungibleBucket`: The new admin badge.
        pub fn new_admin_badge(&mut self) -> NonFungibleBucket {
            // Ensure that a new user badge can be minted
            assert!(self.services.get(PlatformService::MintBadge).value, "PlatformService::MintBadge disabled");
            assert!(self.admin_count < u64::MAX, "Cannot mint more user badges; at U64 MAX");

            // Create empty user badge
            let badge_id = NonFungibleLocalId::Integer(self.admin_count.into());

            // Increment user badge count
            self.admin_count += 1;

            self.admin_badge_manager.mint_non_fungible(&badge_id, ())
        }

        //] Services
        /// Updates a platform service, assuming it is not locked.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        ///
        /// # Panics
        /// - If the service is currently locked.
        pub fn update_service(&mut self, service: PlatformService, value: bool) {
            self.services.update(service, value, false);
        }

        /// Updates a platform service and sets the lock state.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        /// - `locked`: The value to which the lock status of the service is set to.
        pub fn update_service_and_set_lock(&mut self, service: PlatformService, value: bool, locked: bool) {
            self.services.update(service, value, locked);
        }
    }
}
