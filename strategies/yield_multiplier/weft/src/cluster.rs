/* ------------------ Imports ----------------- */
use crate::integration::*;
use scrypto::prelude::*;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod yield_multiplier_weft_v2_cluster {
    //] --------------- Scrypto Setup -------------- /
    // enable_method_auth! {
    //     roles {
    //         platform => updatable_by: [OWNER];
    //     },
    //     methods {
    //         // Platform
    //         get_parent_platform => PUBLIC;
    //         add_verification    => restrict_to: [SELF, OWNER];
    //         // Positions
    //         add_unused_position => PUBLIC;
    //     }
    // }

    //] ------------- Cluster Blueprint ------------ /

    struct YieldMultiplierV1ClusterWeftV2 {
        links: KeyValueStore<ComponentAddress, Vault>,
    }

    impl YieldMultiplierV1ClusterWeftV2 {
        pub fn instantiate() -> (Global<YieldMultiplierV1ClusterWeftV2>, Bucket) {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierV1ClusterWeftV2::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Platform
            // let platform_access_rule: AccessRule = rule!(require(global_caller(parent_platform)));

            // Component owner
            let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_NONE)
                .metadata(metadata! {init {
                    "name"        => "Lattic3 Owner Badge", locked;
                    "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
                }})
                .mint_initial_supply(1)
                .into();
            let owner_access_rule: AccessRule = rule!(require(owner_badge.resource_address()));
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_access_rule.clone());

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
                    "name"            => "L3//Cluster - Yield Multiplier V1@WeftV2", locked;
                    "description"     => "Lattic3 cluster component for the 'Yield Multiplier v1' strategy, built on top of the Weft V2 lending platform.", locked;
                    // "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            // let component_roles = roles! {
            //     platform => platform_access_rule.clone();
            // };

            // Instantisation
            let initial_state = Self {};

            let component: Global<YieldMultiplierV1ClusterWeftV2> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                // .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            (component, owner_badge)
        }

        //] ----------------- Platform ----------------- */
        pub fn handle_link(&mut self, bucket: NonFungibleBucket) -> Result<(), String> {
            // Ensure that the platform hasn't already been linked
            if self.links.get(&cluster_address).is_some() {
                return Err("Platform already linked".to_string());
            }

            // Only allow a single badge to be linked
            if bucket.amount() != 1 {
                return Err("One badge has to be linked".to_string());
            }

            self.links.insert(cluster_address, Vault::from(bucket));

            Ok(())
        }

        //] ----------------- Positions ---------------- */
        pub fn add_unused_position(&mut self, position: NonFungibleBucket) {}
    }
}
