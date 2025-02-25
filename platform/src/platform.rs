/* ------------------ Imports ----------------- */
use crate::clusters::ClusterWrapper;
use crate::services::cluster_services::ClusterService;
use scrypto::prelude::*;
use shared::links::Link;
use shared::users::{User, UserBadge};

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod platform {
    //] --------------- Scrypto Setup -------------- */
    enable_method_auth! {
        roles {
            can_manage_links => updatable_by: [OWNER];
        },
        methods {
            // User
            new_user      => PUBLIC;
            open_account  => PUBLIC; // Restricted by link badge
            close_account => PUBLIC; // Restricted by link badge
            // Links
            link_cluster   => restrict_to: [can_manage_links, OWNER, SELF];
            unlink_cluster => restrict_to: [can_manage_links, OWNER, SELF];
            update_cluster_service => restrict_to: [can_manage_links, OWNER, SELF];
            // State
            get_user_badge_address => PUBLIC;
        }
    }

    //] ------------ Platform Blueprint ------------ */
    struct Platform {
        // Authorisation
        component_address: ComponentAddress,
        // User badges
        user_badge_manager: NonFungibleResourceManager,
        user_count: u64,
        // Links
        link_badge_manager: NonFungibleResourceManager,
        linked_clusters: KeyValueStore<ComponentAddress, ClusterWrapper>,
        linked_count: u64,
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

            //] User badges
            let user_badge_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<User>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//User Badge", locked;
                    "description"     => "Badge used to denote a user's ownership over accounts in Lattic3 clusters.", locked;
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
                .create_with_no_initial_supply();

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
            let component_roles = roles! {
                can_manage_links => OWNER;
            };

            // Instantisation
            let initial_state = Self {
                // Authorisation
                component_address,
                // User badges
                user_badge_manager,
                user_count: 0,
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

            (component, owner_badge)
        }

        //] ------------------- User ------------------- */
        pub fn new_user(&mut self) -> NonFungibleBucket {
            // TODO: integrate own service

            // Ensure that a new user badge can be minted
            assert!(self.user_count < u64::MAX, "Cannot mint more user badges; at U64 MAX");

            // Create empty user badge
            let badge_data: User = User::new();
            let badge_id = NonFungibleLocalId::Integer(self.user_count.into());

            // Increment user badge count
            self.user_count += 1;

            self.user_badge_manager.mint_non_fungible(&badge_id, badge_data)
        }

        pub fn open_account(&self, link_badge: NonFungibleProof, user_badge: UserBadge) {
            // TODO: integrate own service

            // Validate the link
            let wrapper = self.__validate_link(link_badge);
            let can_update_badge = wrapper.services.get_service(ClusterService::UpdateBadge).value;
            assert_eq!(can_update_badge, true, "ClusterService::UpdateBadge disabled");

            // Validate the user badge
            let valid_user = match user_badge {
                UserBadge::Raw(proof) => self.__validate_user(proof),
                UserBadge::Valid(valid_user) => valid_user,
            };

            // Open account and update badge
            let local_id = &valid_user.non_fungible_local_id();
            let mut user: User = valid_user.non_fungible().data();
            user.add_account(wrapper.cluster_address);

            self.user_badge_manager
                .update_non_fungible_data(local_id, "accounts_in", user.accounts_in);
            self.user_badge_manager.update_non_fungible_data(local_id, "open", user.open);
        }

        pub fn close_account(&self, link_badge: NonFungibleProof, user_badge: UserBadge) {
            // TODO: integrate own service

            // Validate the link
            let wrapper = self.__validate_link(link_badge);
            let can_update_badge = wrapper.services.get_service(ClusterService::UpdateBadge).value;
            assert_eq!(can_update_badge, true, "ClusterService::UpdateBadge disabled");

            // Validate the user badge
            let valid_user = match user_badge {
                UserBadge::Raw(proof) => self.__validate_user(proof),
                UserBadge::Valid(valid_user) => valid_user,
            };

            // Open account and update badge
            let local_id = &valid_user.non_fungible_local_id();
            let mut user: User = valid_user.non_fungible().data();
            user.remove_account(wrapper.cluster_address);

            self.user_badge_manager
                .update_non_fungible_data(local_id, "accounts_in", user.accounts_in);
            self.user_badge_manager.update_non_fungible_data(local_id, "open", user.open);
        }

        //] Private
        fn __validate_user(&self, user_badge: NonFungibleProof) -> CheckedNonFungibleProof {
            let valid_user = user_badge.check_with_message(self.user_badge_manager.address(), "User badge not valid");
            assert_eq!(valid_user.amount(), dec!(1), "Invalid user badge quantity");

            valid_user
        }

        //] ------------------- Links ------------------ */
        // RESTRICT can_link/component
        pub fn link_cluster(&mut self, cluster_address: ComponentAddress, package_address: PackageAddress) {
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
            let wrapper = ClusterWrapper::new(cluster_address, package_address, link_id);

            // Deposit badge into cluster and insert into KV
            wrapper.call::<()>("handle_link", scrypto_args!(link_badge));
            self.linked_clusters.insert(cluster_address, wrapper);
        }

        pub fn unlink_cluster(&mut self, cluster_address: ComponentAddress) {
            let _wrapper = self.linked_clusters.get(&cluster_address).ok_or("Cluster already linked".to_string());

            self.linked_clusters.remove(&cluster_address);
        }

        pub fn update_cluster_service(&mut self, cluster_address: ComponentAddress, service: ClusterService, value: bool) {
            let mut wrapper = self
                .linked_clusters
                .get_mut(&cluster_address)
                .expect("Cluster with given address not linked");
            wrapper.services.update_service(service, value);
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

        //] ------------------- State ------------------ */
        pub fn get_user_badge_address(&self) -> ResourceAddress {
            self.user_badge_manager.address()
        }

        // ! ------------------ Testing ----------------- */
    }
}
