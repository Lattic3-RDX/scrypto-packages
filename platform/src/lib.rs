/* ------------------ Imports ----------------- */
// Libraries
use scrypto::prelude::*;
// Modules
mod strategies;
use strategies::VerifiedStrategy;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod platform {
    //] --------------- Scrypto Setup -------------- /
    enable_method_auth! {
    //     roles {
    //         verified => updatable_by: [OWNER];
    //     },
        methods {
            verify_cluster => restrict_to: [SELF, OWNER];
        }
    }

    extern_blueprint! {
        // Using the XRD package to allow a generic 'any Cluster' struct
        // This is very much a hack, but Radix doesn't offer a better way for now
        "package_rdx1pkgxxxxxxxxxpackgexxxxxxxxx000726633226xxxxxxxxxpackge",
        Cluster {
            fn get_parent_platform(&self) -> ComponentAddress;
            fn add_verification(&self, verification_badge: NonFungibleBucket);
        }
    }

    //] ------------- Platform Blueprint ------------ /

    struct Platform {
        // Authorisation
        owner_badge_address: ResourceAddress,
        component_address: ComponentAddress,
        verification_manager: NonFungibleResourceManager,
        // Clusters
        verified: u64,
        clusters: KeyValueStore<BlueprintId, Vec<ComponentAddress>>,
    }

    impl Platform {
        pub fn instantiate_locally(
            component_address: ComponentAddress,
            // dapp_definition_address: ComponentAddress,
            owner_badge_address: ResourceAddress,
            verification_manager: NonFungibleResourceManager,
        ) -> Owned<Platform> {
            //] Authorisation
            // Component
            // let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            // let owner_access_rule: AccessRule = rule!(require(owner_badge_address));
            // let owner_role: OwnerRole = OwnerRole::Fixed(owner_access_rule.clone());

            //] Component Instantisation
            let initial_state = Self {
                owner_badge_address,
                component_address,
                verification_manager,
                verified: 0,
                clusters: KeyValueStore::new(),
            };

            let component = initial_state.instantiate();

            component
        }

        /// Instantiate a new Lattic3 Central Platform.
        pub fn instantiate(
            dapp_definition_address: ComponentAddress,
            owner_badge: Option<Proof>,
            // verification_manager: Option<NonFungibleResourceManager>,
        ) -> (Global<Platform>, Option<Bucket>) {
            // Reserve address
            let (address_reservation, component_address) = Runtime::allocate_component_address(Platform::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            let owner_badge_address: ResourceAddress;
            let owner_badge_bucket: Option<Bucket>;

            if let Some(owner_badge) = owner_badge {
                owner_badge_bucket = None;
                owner_badge_address = owner_badge.resource_address();
            } else {
                owner_badge_bucket = Some(
                    ResourceBuilder::new_fungible(OwnerRole::None)
                        .divisibility(DIVISIBILITY_NONE)
                        .metadata(metadata! {init {
                            "name"        => "Lattic3 Owner Badge", locked;
                            "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
                        }})
                        .mint_initial_supply(1)
                        .into(),
                );
                owner_badge_address = owner_badge_bucket.as_ref().unwrap().resource_address();
            }

            let owner_rule = rule!(require(owner_badge_address));
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_rule.clone());

            // Verified strategy cluster
            let verification_manager: NonFungibleResourceManager = ResourceBuilder::new_integer_non_fungible::<VerifiedStrategy>(owner_role.clone())
                .metadata(metadata! {init {
                    "name"            => "L3//Verified Cluster", locked;
                    "description"     => "Badge representing a strategy cluster that has been verified by the central platform.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
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
                    "name"            => "L3//Central Platform", locked;
                    "description"     => "Central platform for the Lattic3 strategy aggregator.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            // let component_roles = roles! {
            //     verified => rule!(require(verification_manager.resource_address()));
            // };

            // Instantisation
            let owned_component = Platform::instantiate_locally(component_address, owner_badge_address, verification_manager);
            let component = owned_component
                .prepare_to_globalize(owner_role)
                // .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            // Returns
            (component, owner_badge_bucket)
        }

        /// Verify a strategy cluster.
        pub fn verify_cluster(&mut self, cluster_address: ComponentAddress) {
            let cluster: Global<Cluster> = self.__get_cluster(cluster_address);
            let blueprint_id = cluster.blueprint_id();

            assert_eq!(
                cluster.get_parent_platform(),
                self.component_address,
                "Strategy Cluster expects a different parent platform"
            );
            // TODO: ensure that cluster is not already verified

            // Issue verification badge to component
            // TODO: fetch cluster metadata (name) for the VerifiedStrategy struct
            let verification_data: VerifiedStrategy = VerifiedStrategy { name: "DEFAULT CLUSTER NAME".to_string(), verified_by: cluster_address };
            let verification_badge: NonFungibleBucket = self
                .verification_manager
                .mint_non_fungible(&NonFungibleLocalId::integer(self.verified), verification_data);

            cluster.add_verification(verification_badge);
            self.verified += 1;

            // Insert cluster into KV store
            let value = match self.clusters.get(&blueprint_id) {
                Some(addresses) => {
                    let mut addresses = addresses.clone();
                    addresses.push(cluster_address);
                    addresses
                }
                None => vec![cluster_address],
            };
            self.clusters.insert(blueprint_id, value);
        }

        //] ----------- Internal Methods ------------ /
        /// Uses unnatural methods, see the [Radix docs](https://docs.radixdlt.com/docs/advanced-external-calls)
        /// to get a general `Cluster` struct from a `ComponentAddress`.
        fn __get_cluster(&self, address: ComponentAddress) -> Global<Cluster> {
            Global::<Cluster>(Cluster { handle: ObjectStubHandle::Global(address.into()) })
        }
    }
}
