/* ------------------ Imports ----------------- */
use crate::weft::CDPData;
use scrypto::prelude::*;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod yield_multiplier_weft_v2_cluster {
    //] --------------- Scrypto Setup -------------- /

    //] ------------- Cluster Blueprint ------------ /

    use std::panic::catch_unwind;

    struct YieldMultiplierV1ClusterWeftV2 {
        // Authorisation
        component_address: ComponentAddress,
        owner_address: ResourceAddress,
        // Links
        links: KeyValueStore<ComponentAddress, NonFungibleVault>,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        accounts: KeyValueStore<NonFungibleLocalId, ()>,
        // Integration
        cdp_manager: NonFungibleResourceManager,
    }

    impl YieldMultiplierV1ClusterWeftV2 {
        pub fn instantiate(
            // Authorisation
            owner_proof: FungibleProof,
            // Cluster
            supply: ResourceAddress,
            debt: ResourceAddress,
            // Integration
            cdp_resource: ResourceAddress,
        ) -> Global<YieldMultiplierV1ClusterWeftV2> {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierV1ClusterWeftV2::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Platform
            // let platform_access_rule: AccessRule = rule!(require(global_caller(parent_platform)));

            // Component owner
            let owner_address = owner_proof.resource_address();
            let owner_access_rule: AccessRule = rule!(require(owner_address));
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
            let initial_state = Self {
                component_address,
                owner_address,
                links: KeyValueStore::new(),
                supply,
                debt,
                accounts: KeyValueStore::new(),
                cdp_manager: cdp_resource.into(),
            };

            let component: Global<YieldMultiplierV1ClusterWeftV2> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                // .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }

        //] ------------------- Links ------------------ */
        pub fn handle_link(&mut self, platform: ComponentAddress, bucket: NonFungibleBucket) {
            // Sanity checks
            assert!(self.links.get(&platform).is_none(), "Platform already linked");
            assert_eq!(bucket.amount(), dec!(1), "Invalid bucket amount; must contain 1 link badge");

            // Link platform
            let vault = NonFungibleVault::with_bucket(bucket);
            self.links.insert(platform, vault);
        }

        //] ----------------- Accounts ----------------- */
        //] ------------------- Weft ------------------- */
        pub fn validate_cdp(&self, local_id: NonFungibleLocalId) -> bool {
            // Parse CDP data or return false if fetching the data panics
            // Panic occurs if the cdp_manager cannot find an NFT with a matching local_id
            let cdp: CDPData = match catch_unwind(|| self.cdp_manager.get_non_fungible_data::<CDPData>(&local_id)) {
                Ok(cdp) => cdp,
                Err(_) => {
                    info!("Error parsing CDP with local_id {:?}", local_id);
                    return false;
                }
            };

            // Validate supply & debt amounts
            if cdp.collaterals.len() > 1 {
                info!("CDP with local_id {:?} has more than one collateral", local_id);
                return false;
            }

            if cdp.loans.len() > 1 {
                info!("CDP with local_id {:?} has more than one loan", local_id);
                return false;
            }

            // Validate that there are no NFT collaterals
            if cdp.nft_collaterals.len() != 0 {
                info!("CDP with local_id {:?} has NFT collateral(s)", local_id);
                return false;
            }

            //? Check for (unlikely) invalid CDP states
            // if cdp.loans.len() == 1 && cdp.collaterals.len() == 0 {
            //     info!("Invalid CDP state: 1 loan, 0 collateral");
            //     return false;
            // }

            // Validate that all supply and debt assets are valid
            for (&resource, _) in cdp.collaterals.iter() {
                if resource != self.supply {
                    info!("CDP with local_id {:?} has an invalid supply asset", local_id);
                    return false;
                }
            }

            for (&resource, _) in cdp.loans.iter() {
                if resource != self.debt {
                    info!("CDP with local_id {:?} has an invalid debt asset", local_id);
                    return false;
                }
            }

            true
        }
    }
}
