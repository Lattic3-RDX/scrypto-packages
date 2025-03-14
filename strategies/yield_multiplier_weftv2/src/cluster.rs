/* ------------------ Imports ----------------- */
use crate::execution::ExecutionTerms;
use crate::info::{AccountInfo, ClusterInfo, EventAccountInfo, EventClusterInfo};
use crate::services::{ClusterService, ClusterServiceManager};
use crate::weft::{CDPData, CDPHealthChecker};
use scrypto::prelude::*;
use std::panic::catch_unwind;

/* ----------------- Blueprint ---------------- */
#[blueprint]
#[events(EventAccountInfo, EventClusterInfo)]
mod yield_multiplier_weftv2_cluster {

    //] --------------- Scrypto Setup -------------- */
    enable_method_auth! {
        roles {
            can_collect_fees    => updatable_by: [OWNER];
            can_update_services => updatable_by: [OWNER];
            can_lock_services   => updatable_by: [OWNER];
        },
        methods {
            // Links
            handle_link => PUBLIC;
            // Accounts
            open_account     => PUBLIC;
            close_account    => PUBLIC;
            get_cluster_info => PUBLIC;
            get_account_info => PUBLIC;
            start_execution  => PUBLIC;
            end_execution    => PUBLIC;
            // Internal
            update_service              => restrict_to: [can_update_services, can_lock_services];
            update_service_and_set_lock => restrict_to: [can_lock_services];
            // WeftV2 Integration
            validate_cdp => PUBLIC;
        }
    }

    //] ------------- Cluster Blueprint ------------ */
    struct YieldMultiplierWeftV2Cluster {
        // Authorisation
        component_address: ComponentAddress,
        // Platform link
        platform_address: ComponentAddress,
        link: NonFungibleVault,
        user_resource: ResourceAddress,
        // Cluster
        supply: ResourceAddress,
        debt: ResourceAddress,
        accounts: KeyValueStore<NonFungibleLocalId, NonFungibleVault>,
        account_count: u64,
        execution_term_manager: NonFungibleResourceManager,
        services: ClusterServiceManager,
        // Integration
        cdp_manager: NonFungibleResourceManager,
    }

    impl YieldMultiplierWeftV2Cluster {
        pub fn instantiate(
            // Authorisation
            owner_rule: AccessRule,
            // Link
            platform_address: ComponentAddress,
            link_resource: ResourceAddress,
            user_resource: ResourceAddress,
            // Cluster
            supply: ResourceAddress,
            debt: ResourceAddress,
            // Integration
            cdp_resource: ResourceAddress,
        ) -> Global<YieldMultiplierWeftV2Cluster> {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierWeftV2Cluster::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Platform
            // let platform_access_rule: AccessRule = rule!(require(global_caller(parent_platform)));

            // Component owner
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_rule.clone());

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
            let component_roles = roles! {
                can_collect_fees    => OWNER;
                can_update_services => OWNER;
                can_lock_services   => OWNER;
            };

            // Instantisation
            let initial_state = Self {
                component_address,
                platform_address,
                link: NonFungibleVault::new(link_resource),
                user_resource,
                supply,
                debt,
                accounts: KeyValueStore::new(),
                account_count: 0,
                execution_term_manager,
                services: ClusterServiceManager::new(),
                cdp_manager: cdp_resource.into(),
            };

            let component: Global<YieldMultiplierWeftV2Cluster> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }

        //] ------------------- Links ------------------ */
        pub fn handle_link(&mut self, bucket: NonFungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::Link).value, "ClusterService::Link disabled");

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
        fn __with_link<F: FnOnce(Global<AnyComponent>, NonFungibleProof)>(&self, func: F) {
            // Check operating service
            assert!(self.services.get(ClusterService::CallLinked).value, "ClusterService::CallLinked disabled");

            // Arrange call
            let link_local_id = self.link.non_fungible_local_id();
            let link_badge = self.link.create_proof_of_non_fungibles(&indexset![link_local_id]);

            let platform: Global<AnyComponent> = self.platform_address.into();

            func(platform, link_badge);
        }

        //] ------------------ Cluster ----------------- */
        pub fn get_cluster_info(&self) -> ClusterInfo {
            let info = ClusterInfo {
                platform_address: self.platform_address,
                cluster_address: self.component_address,
                linked: self.link.amount() > dec!(0),
                account_count: self.account_count,
                supply_res: self.supply,
                debt_res: self.debt,
            };

            Runtime::emit_event(EventClusterInfo { info: info.clone() });
            info
        }

        //] Services
        pub fn update_service(&mut self, service: ClusterService, value: bool) {
            self.services.update(service, value, false);
        }

        pub fn update_service_and_set_lock(&mut self, service: ClusterService, value: bool, locked: bool) {
            self.services.update(service, value, locked);
        }

        //] ----------------- Accounts ----------------- */
        pub fn open_account(&mut self, user_badge: NonFungibleProof, cdp: NonFungibleBucket) {
            // Check operating service
            assert!(
                self.services.get(ClusterService::OpenAccount).value,
                "ClusterService::OpenAccount disabled"
            );
            assert!(self.account_count <= u64::MAX, "Accounts at u64::MAX");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the CDP
            assert_eq!(cdp.amount(), dec!(1), "Invalid CDP amount; must contain 1 NFT");
            assert_eq!(cdp.resource_address(), self.cdp_manager.address(), "Invalid CDP resource address");

            let cdp_valid = self.validate_cdp(cdp.non_fungible_local_id());
            assert!(cdp_valid, "Invalid CDP");

            // Update the user's badge
            let valid_user = self.__validate_user(user_badge);
            let user_id = valid_user.non_fungible_local_id();

            assert!(
                self.accounts.get(&user_id).is_none() || self.accounts.get(&user_id).unwrap().amount() == dec!(0),
                "User already has an account"
            );

            self.__with_link(|platform, link_badge| platform.call_raw("open_account", scrypto_args!(link_badge, user_id)));

            // Add the CDP to the cluster
            let cdp_local_id = cdp.non_fungible_local_id();
            let cdp_vault = NonFungibleVault::with_bucket(cdp);
            self.accounts.insert(cdp_local_id, cdp_vault);

            // Update the account count
            self.account_count += 1;
        }

        pub fn close_account(&mut self, user_badge: NonFungibleProof) -> NonFungibleBucket {
            // Check operating service
            assert!(
                self.services.get(ClusterService::CloseAccount).value,
                "ClusterService::CloseAccount disabled"
            );
            assert!(self.account_count > 0, "No accounts to close");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate and update the user's badge
            let valid_user = self.__validate_user(user_badge);
            let user_id = valid_user.non_fungible_local_id();
            self.__with_link(|platform, link_badge| platform.call_raw("close_account", scrypto_args!(link_badge, user_id)));

            // Extract the CDP and remove it from the cluster
            let local_id = valid_user.non_fungible_local_id();
            let cdp_bucket = self.accounts.get_mut(&local_id).expect("User has no open account").take_all();

            self.account_count -= 1;

            cdp_bucket
        }

        pub fn get_account_info(&self, weft_address: ComponentAddress, local_id: NonFungibleLocalId) -> AccountInfo {
            let cdp_id = self.accounts.get(&local_id).expect("User has no open account").non_fungible_local_id();
            let weft: Global<AnyComponent> = weft_address.into();

            // Fetch and parse the CDP
            let cdp = weft.call_raw::<CDPHealthChecker>("get_cdp", scrypto_args!(indexset![cdp_id]));
            let supply = match cdp.collateral_positions.get(&self.supply) {
                Some(collateral) => collateral.amount,
                None => dec!(0),
            };
            let debt = match cdp.collateral_positions.get(&self.debt) {
                Some(loan) => loan.amount,
                None => dec!(0),
            };

            // Construct and emit the account info
            let info = AccountInfo {
                cdp_id: local_id,
                supply,
                supply_value: cdp.total_collateral_value,
                debt,
                debt_value: cdp.total_loan_value,
                health: cdp.health_ltv,
            };

            Runtime::emit_event(EventAccountInfo { info: info.clone() });
            info
        }

        pub fn start_execution(&mut self, user_badge: NonFungibleProof) -> (NonFungibleBucket, NonFungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::Execute).value, "ClusterService::Execute disabled");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the user's badge and get the CDP
            let valid_user = self.__validate_user(user_badge);

            let local_id = valid_user.non_fungible_local_id();
            let cdp_bucket = self.accounts.get_mut(&local_id).expect("User has no open account").take_all();

            assert!(cdp_bucket.amount() == dec!(1), "Invalid CDP amount; must contain 1 NFT");

            // Mint the execution terms
            let execution_terms = ExecutionTerms::new(cdp_bucket.non_fungible_local_id(), local_id);
            let terms_bucket = self.execution_term_manager.mint_ruid_non_fungible(execution_terms);

            // Return CDP and execution terms
            (cdp_bucket, terms_bucket)
        }

        pub fn end_execution(&mut self, user_badge: NonFungibleProof, cdp_bucket: NonFungibleBucket, terms_bucket: NonFungibleBucket) {
            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the user's badge
            let valid_user = self.__validate_user(user_badge);
            let user_id = valid_user.non_fungible_local_id();

            // Validate the execution terms
            assert_eq!(terms_bucket.amount(), dec!(1), "Invalid execution terms amount; must contain 1 NFT");
            assert_eq!(
                terms_bucket.resource_address(),
                self.execution_term_manager.address(),
                "Invalid execution terms resource address"
            );

            let term_data: ExecutionTerms = terms_bucket.non_fungible().data();
            let cdp_id = cdp_bucket.non_fungible_local_id();

            assert_eq!(
                term_data.user_local_id, user_id,
                "Presented user badge does not match the execution terms"
            );
            assert_eq!(term_data.cdp_id, cdp_id, "Presented CDP does not match the execution terms");

            // let local_id = terms_bucket.non_fungible_local_id();
            // let execution_terms: ExecutionTerms = self.execution_term_manager.get_non_fungible_data(&local_id);

            // Validate the cdp
            assert_eq!(cdp_bucket.amount(), dec!(1), "Invalid CDP amount; must contain 1 NFT");
            assert_eq!(cdp_bucket.resource_address(), self.cdp_manager.address(), "Invalid CDP resource address");

            let cdp_valid = self.validate_cdp(cdp_bucket.non_fungible_local_id());
            assert!(cdp_valid, "Invalid CDP");

            // Return the CDP
            self.accounts.get_mut(&user_id).expect("User has no open account").put(cdp_bucket);

            // Burn the terms
            self.execution_term_manager.burn(terms_bucket);
        }

        //] Private
        fn __validate_user(&self, user_badge: NonFungibleProof) -> CheckedNonFungibleProof {
            let valid_user = user_badge.check_with_message(self.user_resource, "User badge not valid");
            assert_eq!(valid_user.amount(), dec!(1), "Invalid user badge quantity");

            valid_user
        }

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
                    info!("CDP with local_id {:?} has an invalid collateral asset", local_id);
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
