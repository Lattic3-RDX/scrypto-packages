/* ------------------ Imports ----------------- */
use crate::execution::ExecutionTerms;
use crate::weft::CDPData;
use scrypto::prelude::*;
use std::panic::catch_unwind;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod yield_multiplier_weft_v2_cluster {
    //] --------------- Scrypto Setup -------------- */
    //] ------------- Cluster Blueprint ------------ */
    struct YieldMultiplierV1ClusterWeftV2 {
        // Authorisation
        component_address: ComponentAddress,
        owner_address: ResourceAddress,
        // Platform link
        platform_address: ComponentAddress,
        link: NonFungibleVault,
        user_badge_address: ResourceAddress,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        accounts: KeyValueStore<NonFungibleLocalId, NonFungibleVault>,
        execution_term_manager: NonFungibleResourceManager,
        // Integration
        cdp_manager: NonFungibleResourceManager,
    }

    impl YieldMultiplierV1ClusterWeftV2 {
        pub fn instantiate(
            // Authorisation
            owner_proof: FungibleProof,
            // Link
            platform_address: ComponentAddress,
            link_resource: ResourceAddress,
            user_badge_address: ResourceAddress,
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

            // Execution term manager
            let execution_term_manager = ResourceBuilder::new_ruid_non_fungible::<ExecutionTerms>(owner_role.clone())
                .mint_roles(mint_roles! {
                    minter         => component_access_rule.clone();
                    minter_updater => rule!(deny_all);
                })
                .burn_roles(burn_roles! {
                    burner         => component_access_rule.clone();
                    burner_updater => rule!(deny_all);
                })
                .deposit_roles(deposit_roles! {
                    depositor         => rule!(deny_all);
                    depositor_updater => rule!(deny_all);
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
                    "name"            => "L3//Cluster - Yield Multiplier V1@WeftV2", locked;
                    "description"     => "Lattic3 cluster component for the 'Yield Multiplier v1' strategy, built on top of the Weft V2 lending platform.", locked;
                    // "dapp_definition" => dapp_definition_address, updatable;
                    "execution_terms" => execution_term_manager.address(), locked;
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
                platform_address,
                link: NonFungibleVault::new(link_resource),
                user_badge_address,
                supply,
                debt,
                accounts: KeyValueStore::new(),
                execution_term_manager,
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
        pub fn handle_link(&mut self, bucket: NonFungibleBucket) {
            // Sanity checks
            assert_eq!(self.link.amount(), dec!(0), "Platform already linked");
            assert_eq!(bucket.amount(), dec!(1), "Invalid bucket amount; must contain 1 link badge");
            assert_eq!(
                self.link.resource_address(),
                bucket.resource_address(),
                "Invalid link badge resource address"
            );

            // Link platform
            self.link.put(bucket);
        }

        //] Private

        //] ------------------ Cluster ----------------- */
        pub fn open_account(&mut self, user_badge: NonFungibleProof, cdp: NonFungibleBucket) {
            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the CDP
            assert_eq!(cdp.amount(), dec!(1), "Invalid CDP amount; must contain 1 NFT");
            assert_eq!(cdp.resource_address(), self.cdp_manager.address(), "Invalid CDP resource address");

            // Update the user's badge
            let link_local_id = self.link.non_fungible_local_id();
            let link_badge = self.link.create_proof_of_non_fungibles(&indexset![link_local_id]);

            let platform: Global<AnyComponent> = self.platform_address.into();
            platform.call_raw::<()>("open_account", scrypto_args!(link_badge, user_badge));

            // Add the CDP to the cluster
            let cdp_local_id = cdp.non_fungible_local_id();
            let cdp_vault = NonFungibleVault::with_bucket(cdp);
            self.accounts.insert(cdp_local_id, cdp_vault);
        }

        pub fn close_account(&mut self, user_badge: NonFungibleProof) {} // -> NonFungibleBucket (CDP)

        // pub fn get_account_info(&self, local_id: NonFungibleLocalId) {}

        pub fn execute(&mut self, user_badge: NonFungibleProof) {} // -> NonFungibleBucket (ExecutionTerms)

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
