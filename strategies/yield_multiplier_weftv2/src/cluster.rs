/* ------------------ Imports ----------------- */
use crate::execution::ExecutionTerms;
use crate::info::{AccountInfo, ClusterInfo, EventAccountInfo, EventClusterInfo};
use crate::services::{ClusterService, ClusterServiceManager};
use crate::weft::{CDPData, CDPHealthChecker};
use scrypto::prelude::*;
use shared::services::SetLock;
use std::panic::catch_unwind;

/* ----------------- Blueprint ---------------- */
#[blueprint]
#[types(
    ComponentAddress,
    ResourceAddress,
    NonFungibleResourceManager,
    NonFungibleLocalId,
    Decimal,
    u64,
    BlueprintId,
    ClusterServiceManager,
    ClusterService,
    AccountInfo,
    ClusterInfo,
    CDPData,
    CDPHealthChecker,
    SetLock
)]
#[events(EventAccountInfo, EventClusterInfo)]
mod yield_multiplier_weftv2_cluster {
    //] --------------- Scrypto Setup -------------- */
    enable_method_auth! {
        roles {
            can_manage_fees     => updatable_by: [OWNER];
            can_update_services => updatable_by: [OWNER];
            can_lock_services   => updatable_by: [OWNER];
        },
        methods {
            // Links
            handle_link => PUBLIC;
            // Cluster
            get_cluster_info => PUBLIC;
            update_service              => restrict_to: [can_update_services, can_lock_services];
            update_service_and_set_lock => restrict_to: [can_lock_services];
            set_fee_percentage => restrict_to: [can_manage_fees];
            collect_fees       => restrict_to: [can_manage_fees];
            // Accounts
            open_account     => PUBLIC;
            close_account    => PUBLIC;
            get_account_info => PUBLIC;
            start_execution  => PUBLIC;
            end_execution    => PUBLIC;
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
        fee_rate: Decimal,
        fee_vault: FungibleVault,
        // Integration
        weft_market_address: ComponentAddress,
        cdp_manager: NonFungibleResourceManager,
    }

    impl YieldMultiplierWeftV2Cluster {
        /// Instantiates a new `YieldMultiplierWeftV2Cluster` component with the specified configuration.
        ///
        /// # Parameters
        /// - `owner_rule`: Access rule defining the owner of the component.
        /// - `platform_address`: The component address of the platform to which this cluster links.
        /// - `link_resource`: Resource address for the link badge used to link this cluster to a platform.
        /// - `user_resource`: Resource address for the user badge required for user operations.
        /// - `supply`: Resource address for the supply asset of the cluster.
        /// - `debt`: Resource address for the debt asset of the cluster.
        /// - `cdp_resource`:Resource address of the WeftV2 CDP NFT.
        ///
        /// # Returns
        /// A globally accessible `YieldMultiplierWeftV2Cluster` component instance.
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
            weft_market_address: ComponentAddress,
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
                    "name"            => "L3//Yield Multiplier - WeftV2", locked;
                    "description"     => "Lattic3 cluster component for the Yield Multiplier strategy, built on top of the Weft V2 lending platform.", locked;
                    "supply"          => supply, locked;
                    "debt"            => debt, locked;
                    "execution_terms" => execution_term_manager.address(), locked;
                    // "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            let component_roles = roles! {
                can_manage_fees     => OWNER;
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
                fee_rate: dec!(0.015),
                fee_vault: FungibleVault::new(XRD),
                weft_market_address,
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
        /// Handles the reception of a link badge. Initiated by the platform's link_cluster() method.
        ///
        /// # Parameters
        /// - `bucket`: The bucket containing the link badge.
        ///
        /// # Panics
        /// - If the Link service is disabled
        /// - If the link badge is invalid (amount != 1, incorrect resource address)
        /// - If the cluster is already linked
        pub fn handle_link(&mut self, bucket: NonFungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::Link), "ClusterService::Link disabled");

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
        /// Call a method on the platform; uses a closure with |platform, link_badge_proof|.
        /// Closure is used due to issues with appending generic scyrpto_args to an already-encoded link badge.
        ///
        /// # Parameters
        /// - `func`: Closure with |platform, link_badge_proof|
        ///
        /// # Panics
        /// - If the cluster is not linked
        /// - If the CallLinked service is disabled
        fn __with_link<F: FnOnce(Global<AnyComponent>, NonFungibleProof)>(&self, func: F) {
            assert!(self.link.amount() > dec!(0), "Cluster not linked");
            assert!(self.services.get(ClusterService::CallLinked), "ClusterService::CallLinked disabled");

            // Arrange call
            let link_local_id = self.link.non_fungible_local_id();
            let link_badge = self.link.create_proof_of_non_fungibles(&indexset![link_local_id]);

            let platform: Global<AnyComponent> = self.platform_address.into();

            func(platform, link_badge);
        }

        //] ------------------ Cluster ----------------- */
        /// Returns general information about the cluster.
        ///
        /// # Emits
        /// - `EventClusterInfo`: contains the same information as the returned `ClusterInfo` struct.
        ///
        /// # Returns
        /// A `ClusterInfo` struct containing the following information:
        /// - `platform_address`: The component address of the platform to which this cluster links.
        /// - `cluster_address`: The component address of the cluster.
        /// - `linked`: A boolean indicating whether the cluster is linked.
        /// - `account_count`: The number of accounts open on the cluster.
        /// - `supply_res`: The resource address of the supply asset.
        /// - `debt_res`: The resource address of the debt asset.
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
        /// Updates a cluster service without setting lock.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        ///
        /// # Panics
        /// - If the service is not enabled in the blueprint
        pub fn update_service(&mut self, service: ClusterService, value: bool) {
            self.services.update(service, value, SetLock::None);
        }

        pub fn update_service_and_set_lock(&mut self, service: ClusterService, value: bool, locked: bool) {
            self.services.update(service, value, SetLock::Update(locked));
        }

        //] Fees
        pub fn set_fee_percentage(&mut self, fee_rate: Decimal) {
            self.fee_rate = fee_rate;
        }

        pub fn collect_fees(&mut self) -> FungibleBucket {
            self.fee_vault.take_all()
        }

        //] ----------------- Accounts ----------------- */
        pub fn open_account(&mut self, user_badge: NonFungibleProof, cdp: NonFungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::OpenAccount), "ClusterService::OpenAccount disabled");
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
            self.__with_link(|platform, link_badge| platform.call_raw("open_account", scrypto_args!(link_badge, user_id.clone())));

            // assert!(
            //     self.accounts.get(&user_id).is_none() || self.accounts.get(&user_id).unwrap().amount() == dec!(0),
            //     "User already has an account"
            // );

            // Add the CDP to the cluster
            if self.accounts.get(&user_id).is_some() {
                let mut vault = self.accounts.get_mut(&user_id).unwrap();
                assert!(vault.amount() == dec!(0), "User already has an account");

                vault.put(cdp);
            } else {
                self.accounts.insert(user_id, NonFungibleVault::with_bucket(cdp));
            }

            // Update the account count
            self.account_count += 1;
        }

        pub fn close_account(&mut self, user_badge: NonFungibleProof) -> NonFungibleBucket {
            // Check operating service
            assert!(self.services.get(ClusterService::CloseAccount), "ClusterService::CloseAccount disabled");
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

        pub fn get_account_info(&self, local_id: NonFungibleLocalId) -> AccountInfo {
            let cdp_id = self.accounts.get(&local_id).expect("User has no open account").non_fungible_local_id();
            let weft_market: Global<AnyComponent> = self.weft_market_address.into();

            // Fetch and parse the CDP
            let cdp_health = weft_market.call_raw::<CDPHealthChecker>("get_cdp", scrypto_args!(indexset![cdp_id]));
            let supply = match cdp_health.collateral_positions.get(&self.supply) {
                Some(collateral) => collateral.amount,
                None => dec!(0),
            };
            let debt = match cdp_health.collateral_positions.get(&self.debt) {
                Some(loan) => loan.amount,
                None => dec!(0),
            };

            // Construct and emit the account info
            let info = AccountInfo {
                cdp_id: local_id,
                supply,
                supply_value: cdp_health.total_collateral_value,
                debt,
                debt_value: cdp_health.total_loan_value,
                health: cdp_health.health_ltv,
            };

            Runtime::emit_event(EventAccountInfo { info: info.clone() });
            info
        }

        pub fn start_execution(&mut self, user_badge: NonFungibleProof) -> (NonFungibleBucket, NonFungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::Execute), "ClusterService::Execute disabled");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the user's badge and get the CDP
            let valid_user = self.__validate_user(user_badge);

            let local_id = valid_user.non_fungible_local_id();
            let cdp_bucket = self.accounts.get_mut(&local_id).expect("User has no open account").take_all();

            assert!(cdp_bucket.amount() == dec!(1), "Invalid CDP amount; must contain 1 NFT");

            // Get the CDPHealthChecker for the CDP and get the net total value (liquidity)
            let cdp_id = cdp_bucket.non_fungible_local_id();
            let weft_market: Global<AnyComponent> = self.weft_market_address.into();
            let cdp_health = weft_market.call_raw::<CDPHealthChecker>("get_cdp", scrypto_args!(indexset![cdp_id]));

            let liquidity = cdp_health.total_collateral_value.checked_sub(cdp_health.total_loan_value).unwrap();

            // Mint the execution terms
            let execution_terms = ExecutionTerms::new(cdp_bucket.non_fungible_local_id(), local_id, liquidity);
            let terms_bucket = self.execution_term_manager.mint_ruid_non_fungible(execution_terms);

            // Return CDP and execution terms
            (cdp_bucket, terms_bucket)
        }

        pub fn end_execution(
            &mut self,
            user_badge: NonFungibleProof,
            cdp_bucket: NonFungibleBucket,
            terms_bucket: NonFungibleBucket,
            mut fee_payment: FungibleBucket,
        ) -> FungibleBucket {
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

            let cdp_id = cdp_bucket.non_fungible_local_id();
            let cdp_valid = self.validate_cdp(cdp_id.clone());
            assert!(cdp_valid, "Invalid CDP");

            // Get the CDPHealthChecker for the CDP and get the net total value (liquidity)
            let weft_market: Global<AnyComponent> = self.weft_market_address.into();
            let cdp_health = weft_market.call_raw::<CDPHealthChecker>("get_cdp", scrypto_args!(indexset![cdp_id]));

            let liquidity = cdp_health.total_collateral_value.checked_sub(cdp_health.total_loan_value).unwrap();
            let liquidity_delta = liquidity.checked_sub(term_data.cdp_liquidity).unwrap().checked_abs().unwrap();

            // Validate the fee payment
            let fee_amount = liquidity_delta.checked_mul(self.fee_rate).unwrap();

            assert_eq!(fee_payment.resource_address(), XRD, "Fee repayment must be in XRD");
            assert!(fee_payment.amount() >= fee_amount, "Fee repayment is less than calculated");

            self.fee_vault.put(fee_payment.take(fee_amount));

            // Return the CDP
            self.accounts.get_mut(&user_id).expect("User has no open account").put(cdp_bucket);

            // Burn the terms
            self.execution_term_manager.burn(terms_bucket);

            // Return excess fee repayment
            fee_payment
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
