/* ------------------ Imports ----------------- */
// Libraries
use scrypto::prelude::*;
// Modules
mod integration;
use integration::*;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod yield_multiplier_weft_v2_cluster {
    use scrypto::{
        math::Decimal,
        prelude::{FungibleProof, KeyValueStore},
        runtime::NonFungibleLocalId,
    };

    //] --------------- Scrypto Setup -------------- /
    enable_method_auth! {
        roles {
            platform => updatable_by: [OWNER];
        },
        methods {
            // Platform
            get_parent_platform => PUBLIC;
            add_verification    => restrict_to: [SELF, OWNER];
            // Positions
            add_unused_position => PUBLIC;
        }
    }

    //] ------------- Cluster Blueprint ------------ /

    struct YieldMultiplierV1Cluster {
        // Authorisation
        owner_badge_res: ResourceAddress,           // Resource address of the owner badge
        parent_platform: ComponentAddress, // Component address of the central platform. Must match the one defined in the verification badge.
        verification_badge_vault: NonFungibleVault, // Vault containing the verification badge issued by the central platform.
        // Strategy
        supply_res: ResourceAddress, // Resource address of the supply on the lending platform
        debt_res: ResourceAddress,   // Resource address of the debt on the lending platform
        // Position
        position_res: ResourceAddress,    // Resource address of the lending platform's position badge
        position_vault: NonFungibleVault, // Vault containing the position badges
        unused_positions: KeyValueStore<Decimal, NonFungibleLocalId>, // Lazy-loaded KV store, holding the local ids of unused/closed positions
        unused_positions_count: Decimal,  // Length of unused_positions
        // Admin
        fee_vault: FungibleVault, // Collects fees in `supply_res`
    }

    impl YieldMultiplierV1Cluster {
        pub fn instantiate(
            // Metadata
            dapp_definition_address: ComponentAddress,
            // Authorisation
            owner_badge_proof: FungibleProof,
            parent_platform: ComponentAddress,
            verification_badge_res: ResourceAddress,
            // Strategy parameters
            supply_res: ResourceAddress,
            debt_res: ResourceAddress,
            position_res: ResourceAddress,
        ) -> Global<YieldMultiplierV1Cluster> {
            // Reserve component address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierV1Cluster::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Platform
            let platform_access_rule: AccessRule = rule!(require(global_caller(parent_platform)));

            // Component owner
            // let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            //     .divisibility(DIVISIBILITY_NONE)
            //     .metadata(metadata! {init {
            //         "name"        => "Lattic3 Owner Badge", locked;
            //         "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
            //     }})
            //     .mint_initial_supply(1)
            //     .into();
            let owner_access_rule: AccessRule = rule!(require(owner_badge_proof.resource_address()));
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
                    "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            let component_roles = roles! {
                platform => platform_access_rule.clone();
            };

            // Instantisation
            let initial_state = Self {
                owner_badge_res: owner_badge_proof.resource_address(),
                parent_platform,
                verification_badge_vault: NonFungibleVault::new(verification_badge_res),
                supply_res,
                debt_res,
                position_res,
                position_vault: NonFungibleVault::new(position_res),
                unused_positions: KeyValueStore::new(),
                unused_positions_count: dec!(0),
                fee_vault: FungibleVault::new(supply_res),
            };

            let component: Global<YieldMultiplierV1Cluster> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }

        //] ---------------- Platform --------------- /

        pub fn get_parent_platform(&self) -> ComponentAddress {
            // TODO: Make option, or panic if unverified

            self.parent_platform
        }

        pub fn add_verification(&mut self, verification_badge: NonFungibleBucket) {
            assert_eq!(
                verification_badge.resource_address(),
                self.verification_badge_vault.resource_address(),
                "Incorrect verification badge passed."
            );

            assert_eq!(verification_badge.amount(), dec!(1), "Incorrect amount of verification badge(s) passed.");

            // TODO: assert that verification badge data is expected

            self.verification_badge_vault.put(verification_badge);
        }

        //] --------------- Positions --------------- /

        pub fn add_unused_position(&mut self, position: NonFungibleBucket) {
            assert_eq!(position.resource_address(), self.position_res, "Incorrect position resource passed.");

            assert_eq!(position.amount(), dec!(1), "Incorrect amount of position(s) passed.");

            // TODO: assert that position data is empty

            let local_id = position.non_fungible_local_id();

            // Modify unused position KV
            self.unused_positions.insert(self.unused_positions_count, local_id);
            self.unused_positions_count = self
                .unused_positions_count
                .checked_add(1)
                .expect("Failed to increment unused_positions_count.");

            // Deposit position into vault
            self.position_vault.put(position);
        }

        //] ----------- WeftV2 Integration ---------- /

        //] ------------ Internal Methods ----------- /
    }
}
