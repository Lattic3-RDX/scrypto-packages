/* ------------------ Imports ----------------- */
// Modules
use crate::accounts::AccountData;
use crate::fees::FeeStructure;
use crate::info::{AccountInfo, ClusterInfo, FeeInfo};
use crate::services::{ClusterService, ClusterServiceManager};
use crate::weft::*;
// Shared Modules
use shared::services::{ServiceValue, SetLock};
// use shared::utils::{now, SECONDS_PER_YEAR};
// Libraries
use scrypto::prelude::*;
use std::panic::catch_unwind;

/* ----------------- Blueprint ---------------- */
type Unit = ();

#[derive(NonFungibleData, ScryptoSbor)]
pub struct ExecutionTerms {
    pub user_id: NonFungibleLocalId,
}

#[blueprint]
#[types(
    // General
    Unit,
    ComponentAddress,
    ResourceAddress,
    NonFungibleResourceManager,
    NonFungibleLocalId,
    NonFungibleVault,
    FungibleVault,
    Decimal,
    u64,
    i64,
    FeeStructure,
    ExecutionTerms,
    // Services
    ClusterServiceManager,
    ClusterService,
    ServiceValue,
    SetLock,
    // State Returns
    AccountInfo,
    ClusterInfo,
    FeeInfo,
    // WeftV2 Integration
    CDPData,
    CDPHealthChecker,
    LoanPositionData,
    LoanConfig,
    LoanResourceConfig,
    CollateralPositionData,
    CollateralConfig,
    CollateralResourceConfig,
    RegisteredResourceType,
    NFTCollateralPositionData,
    NFTLiquidationValue,
    RegisteredNFTResourceType,
    EfficiencyMode,
    CollateralConfigVersion,
    CollateralInfo,
    NFTCollateralInfo,
    LoanInfo
)]
mod yield_multiplier_weftv2_cluster {
    //] --------------- Scrypto Setup -------------- */
    enable_method_auth! {
        roles {
            can_manage_services => updatable_by: [OWNER];
            can_lock_services   => updatable_by: [OWNER];
            can_manage_fees     => updatable_by: [OWNER];
        },
        methods {
            // Links
            handle_link => PUBLIC;
            // Cluster
            get_cluster_info => PUBLIC;
            update_service              => restrict_to: [can_manage_services, can_lock_services];
            update_service_and_set_lock => restrict_to: [can_lock_services];
            set_fee_structure           => restrict_to: [can_manage_fees];
            collect_fees                => restrict_to: [can_manage_fees];
            // Accounts
            open_account     => PUBLIC;
            close_account    => PUBLIC;
            get_account_info => PUBLIC;
            start_execution  => PUBLIC;
            end_execution    => PUBLIC;
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
        services: ClusterServiceManager,
        execution_term_manager: NonFungibleResourceManager,
        // Accounts
        accounts: KeyValueStore<NonFungibleLocalId, AccountData>,
        account_count: u64,
        // Fees
        fee_structure: FeeStructure,
        fee_vault: FungibleVault,
        // Integration
        weft_market_address: ComponentAddress,
        cdp_manager: NonFungibleResourceManager,
    }

    impl YieldMultiplierWeftV2Cluster {
        /// Instantiates a new `YieldMultiplierWeftV2Cluster` component with the specified configuration.
        ///
        /// # Parameters
        /// - `owner_rule`: Access rule defining the owner of the cluster.
        /// - `admin_rule`: Access rule defining the admins of the cluster.
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
            admin_rule: AccessRule,
            // Link
            platform_address: ComponentAddress,
            // Cluster
            supply: ResourceAddress,
            debt: ResourceAddress,
            // Integration
            weft_market_address: ComponentAddress,
            cdp_resource: ResourceAddress,
        ) -> Global<YieldMultiplierWeftV2Cluster> {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierWeftV2Cluster::blueprint_id());

            let platform: Global<AnyComponent> = platform_address.into();
            let link_resource = platform.call_raw::<ResourceAddress>("get_link_badge_address", scrypto_args!());
            let user_resource = platform.call_raw::<ResourceAddress>("get_user_badge_address", scrypto_args!());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            let owner_role: OwnerRole = OwnerRole::Fixed(owner_rule.clone());

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
                    // "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            let component_roles = roles! {
                can_manage_fees     => OWNER;
                can_manage_services => admin_rule;
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
                fee_structure: FeeStructure::default(),
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
        /// # Returns
        /// A `ClusterInfo` struct containing the following information:
        /// - `platform_address`: The component address of the platform to which this cluster links.
        /// - `cluster_address`: The component address of the cluster.
        /// - `linked`: A boolean indicating whether the cluster is linked.
        /// - `account_count`: The number of accounts open on the cluster.
        /// - `supply_res`: The resource address of the supply asset.
        /// - `debt_res`: The resource address of the debt asset.
        pub fn get_cluster_info(&self) -> ClusterInfo {
            // Return the fee amounts
            let fee_info = FeeInfo {
                open: self.fee_structure.open,
                close: self.fee_structure.close,
                execute: self.fee_structure.execute,
            };

            let info = ClusterInfo {
                platform_address: self.platform_address,
                cluster_address: self.component_address,
                linked: self.link.amount() > dec!(0),
                supply_res: self.supply,
                debt_res: self.debt,
                account_count: self.account_count,
                execution_term_manager: self.execution_term_manager,
                fee_info,
            };

            info
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
        pub fn update_service(&mut self, service: ClusterService, value: bool) {
            self.services.update(service, value, SetLock::None);
        }

        /// Updates a cluster service and sets the lock state.
        ///
        /// # Parameters
        /// - `service`: The service to update.
        /// - `value`: The value to set the service to.
        /// - `locked`: The value to which the lock status of the service is set to.
        pub fn update_service_and_set_lock(&mut self, service: ClusterService, value: bool, locked: bool) {
            self.services.update(service, value, SetLock::Update(locked));
        }

        //] Fees
        /// Sets a new fee structure for the cluster.
        /// All fees are set in XRD.
        ///
        /// # Parameters
        /// - `open`: Fees for opening an account.
        /// - `close`: Fees for closing an account.
        /// - `execute`: Fees for executing a transaction.
        pub fn set_fee_structure(&mut self, open: Option<Decimal>, close: Option<Decimal>, execute: Option<Decimal>) {
            self.fee_structure.set(open, close, execute);
        }

        /// Collects fees from the fee vault.
        ///
        /// # Returns
        /// - A `FungibleBucket` containing the collected fees.
        pub fn collect_fees(&mut self) -> FungibleBucket {
            self.fee_vault.take_all()
        }

        //] ----------------- Accounts ----------------- */
        /// Opens an account for a user on the cluster. Deposits CDP into the `accounts` KV, in a corresponding vault.
        ///
        /// # Parameters
        /// - `user_badge`: Proof of the user's badge from the platform.
        /// - `cdp`: Weft CDP input.
        ///
        /// # Panics
        /// - If the cluster is not linked.
        /// - If the ClusterService::OpenAccount is disabled.
        /// - If the user already has an account.
        /// - If the CDP is invalid.
        ///
        /// # Returns
        /// A `FungibleBucket` containing the remainder of the fee.
        pub fn open_account(&mut self, user_badge: NonFungibleProof, cdp: NonFungibleBucket, mut fee_payment: FungibleBucket) -> FungibleBucket {
            // Check operating service
            assert!(self.services.get(ClusterService::OpenAccount), "ClusterService::OpenAccount disabled");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the CDP
            assert_eq!(cdp.amount(), dec!(1), "Invalid CDP amount; must contain 1 NFT");
            assert_eq!(cdp.resource_address(), self.cdp_manager.address(), "Invalid CDP resource address");

            let cdp_id = cdp.non_fungible_local_id();
            let cdp_valid = self.__validate_cdp(cdp_id.clone());
            assert!(cdp_valid, "Invalid CDP");

            // Take fee payment
            let fee = self.fee_structure.open;
            self.fee_vault.put(fee_payment.take(fee));

            // Update the user's badge
            let valid_user = self.__validate_user(user_badge);
            let user_id = valid_user.non_fungible_local_id();
            self.__with_link(|platform, link_badge| platform.call_raw("open_account", scrypto_args!(link_badge, user_id.clone())));

            if self.accounts.get(&user_id).is_some() {
                let mut account = self.accounts.get_mut(&user_id).unwrap();
                assert!(account.cdp_vault.amount() == dec!(0), "User already has an account");

                account.cdp_vault.put(cdp);
            } else {
                let account = AccountData::new(NonFungibleVault::with_bucket(cdp));
                self.accounts.insert(user_id, account);
            }

            // Update the account count
            self.account_count += 1;
            fee_payment
        }

        /// Closes an account for a user on the cluster, and withdraws CDP.
        ///
        /// # Parameters
        /// - `user_badge`: Proof of the user's badge from the platform.
        /// - `fee_payment`: A `FungibleBucket` containing the payment for closing the account.
        ///
        /// # Panics
        /// - If the cluster is not linked.
        /// - If the ClusterService::CloseAccount is disabled.
        /// - If the user does not have an account.
        /// - If the fee payment is invalid (wrong type, insufficient amount).
        ///
        /// # Returns
        /// - A `NonFungibleBucket` containing the CDP.
        /// - A `FungibleBucket` containing the remainder of the fee.
        pub fn close_account(&mut self, user_badge: NonFungibleProof, mut fee_payment: FungibleBucket) -> (NonFungibleBucket, FungibleBucket) {
            // Check operating service
            assert!(self.services.get(ClusterService::CloseAccount), "ClusterService::CloseAccount disabled");
            assert!(self.account_count > 0, "No accounts to close");

            // Validate own link badge
            assert_eq!(self.link.amount(), dec!(1), "Cluster does not have a link badge");

            // Validate the user
            let valid_user = self.__validate_user(user_badge);
            let user_id = valid_user.non_fungible_local_id();

            // Validate the fee
            let fee = self.fee_structure.close;
            self.fee_vault.put(fee_payment.take(fee));

            // Extract the CDP and remove it from the cluster
            let cdp_bucket = self.accounts.get_mut(&user_id).expect("User has no open account").cdp_vault.take_all();

            // Update the user's badge
            self.__with_link(|platform, link_badge| platform.call_raw("close_account", scrypto_args!(link_badge, user_id)));

            // Remove the account
            self.account_count -= 1;
            (cdp_bucket, fee_payment)
        }

        /// Returns general information about an account. Queried from Weft using their `get_cdp` method.
        ///
        /// # Parameters
        /// - `local_id`: The local ID of the account to query.
        ///
        /// # Returns
        /// - A `AccountInfo` struct with the account's information.
        pub fn get_account_info(&self, local_id: NonFungibleLocalId) -> AccountInfo {
            let account = self.accounts.get(&local_id).expect("User has no open account");

            // Fetch and parse the CDP
            let cdp_id = account.cdp_vault.non_fungible_local_id();
            let weft_market: Global<AnyComponent> = self.weft_market_address.into();

            let cdp_health_map =
                weft_market.call_raw::<IndexMap<NonFungibleLocalId, CDPHealthChecker>>("get_cdp", scrypto_args!(indexset![cdp_id.clone()]));
            let cdp_health = cdp_health_map.get(&cdp_id.clone()).unwrap();

            let supply = match cdp_health.collateral_positions.get(&self.supply) {
                Some(collateral) => collateral.amount,
                None => dec!(0),
            };
            let debt = match cdp_health.loan_positions.get(&self.debt) {
                Some(loan) => loan.amount,
                None => dec!(0),
            };

            // Construct and emit the account info
            let info = AccountInfo {
                cdp_id,
                supply,
                supply_value: cdp_health.total_collateral_value,
                debt,
                debt_value: cdp_health.total_loan_value,
                health: cdp_health.liquidation_ltv,
            };

            info
        }

        /// Starts an execution on the cluster, allowing the user to perform arbitrary
        /// interactions with the user's CDP. All operations must be executed within
        /// one transaction, and the CPD must be returned to the user at the end by
        /// calling the `end_execution` method.
        ///
        /// # Parameters
        /// - `user_badge`: A `NonFungibleProof` of the user's badge.
        ///
        /// # Panics
        /// - If the cluster is not linked to the platform.
        /// - If the ClusterService::Execute is disabled.
        /// - If the user does not have an open account.
        ///
        /// # Returns
        /// - A `NonFungibleBucket` containing the user's CDP.
        /// - A `NonFungibleBucket` containing the execution terms transient badge.
        pub fn start_execution(&mut self, user_badge: NonFungibleProof) -> (NonFungibleBucket, NonFungibleBucket) {
            // Check ClusterService::Execute enabled
            assert!(self.services.get(ClusterService::Execute), "ClusterService::Execute disabled");

            // Validate the user
            let user_id = self.__validate_user(user_badge).non_fungible_local_id();

            // Return CDP and execution terms
            let cdp_bucket = self.accounts.get_mut(&user_id).expect("User has no open account").cdp_vault.take_all();

            let terms = ExecutionTerms { user_id };
            let execution_terms = self.execution_term_manager.mint_ruid_non_fungible(terms);

            (cdp_bucket, execution_terms)
        }

        /// Counterpart to `start_execution`, returns the user's CDP to the cluster
        /// and validates that the state of the CDP is valid.
        ///
        /// # Parameters
        /// - `cdp_bucket`: A `NonFungibleBucket` containing the user's CDP.
        /// - `terms_bucket`: A `NonFungibleBucket` containing the execution terms transient badge.
        ///
        /// # Panics
        /// - If the user does not have an open account.
        /// - If the CDP is invalid (wrong type, insufficient amount).
        ///
        /// # Returns
        /// - A `FungibleBucket` containing the remainder of the fee.
        pub fn end_execution(
            &mut self,
            cdp_bucket: NonFungibleBucket,
            terms_bucket: NonFungibleBucket,
            mut fee_payment: FungibleBucket,
        ) -> FungibleBucket {
            // Validate the execution terms
            assert!(self.execution_term_manager.address() == terms_bucket.resource_address());
            let terms = terms_bucket.non_fungible::<ExecutionTerms>().data();

            // Validate the CDP
            let cdp_id = cdp_bucket.non_fungible_local_id();
            let cdp_valid = self.__validate_cdp(cdp_id.clone());
            assert!(cdp_valid, "Invalid CDP");

            // Calculate the fee
            let fee = self.fee_structure.execute;
            self.fee_vault.put(fee_payment.take(fee));

            // Return the CDP
            self.accounts
                .get_mut(&terms.user_id)
                .expect("User has no open account")
                .cdp_vault
                .put(cdp_bucket);

            // Burn the execution terms
            self.execution_term_manager.burn(terms_bucket);

            fee_payment
        }

        //] Private
        /// Validates the user's badge and returns the checked proof.
        fn __validate_user(&self, user_badge: NonFungibleProof) -> CheckedNonFungibleProof {
            // assert_eq!(user_badge.resource_address(), self.user_resource, "Invalid user badge resource address");

            let valid_user = user_badge.check_with_message(self.user_resource, "User badge not valid");
            assert_eq!(valid_user.amount(), dec!(1), "Invalid user badge quantity");

            valid_user
        }

        //] ------------------- Weft ------------------- */
        /// Validates the given CDP by checking its contents.
        ///
        /// # Parameters
        /// - `local_id`: The local ID of the CDP to validate.
        ///
        /// # Returns
        /// - `true` if the CDP is valid; otherwise, `false`.
        ///
        /// # Validation Criteria
        /// - The CDP must have a valid ResourceAddress.
        /// - The CDP must contain a maximum of one collateral asset and one loan asset.
        /// - The CDP must not have any NFT collaterals.
        /// - The collateral asset in the CDP must match the expected supply asset.
        /// - The debt asset in the CDP must match the expected debt asset.
        fn __validate_cdp(&self, local_id: NonFungibleLocalId) -> bool {
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
            // for (&resource, _) in cdp.collaterals.iter() {
            //     if resource != self.supply {
            //         info!("CDP with local_id {:?} has an invalid collateral asset", local_id);
            //         return false;
            //     }
            // }
            if cdp.collaterals.len() == 1 {
                if !cdp.collaterals.contains_key(&self.supply) {
                    info!("CDP with local_id {:?} has an invalid collateral asset", local_id);
                    return false;
                }
            }

            // for (&resource, _) in cdp.loans.iter() {
            //     if resource != self.debt {
            //         info!("CDP with local_id {:?} has an invalid debt asset", local_id);
            //         return false;
            //     }
            // }
            if cdp.loans.len() == 1 {
                if !cdp.loans.contains_key(&self.debt) {
                    info!("CDP with local_id {:?} has an invalid debt asset", local_id);
                    return false;
                }
            }

            true
        }
    }
}
