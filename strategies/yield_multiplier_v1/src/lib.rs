/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* ----------------- Blueprint ---------------- */
#[blueprint]
mod yield_multiplier_v1_cluster {
    //] --------------- Scrypto Setup -------------- /
    enable_method_auth! {
        roles {
            platform => updatable_by: [OWNER];
        },
        methods {
            verify_cluster => restrict_to: [SELF, OWNER];
        }
    }

    //] ------------- Cluster Blueprint ------------ /

    struct YieldMultiplierV1 {
        // Authorisation
        owner_badge_address: ResourceAddress,
        parent_platform: ComponentAddress,
        // Strategy
        supply: ResourceAddress,
        debt: ResourceAddress,

        // Position
        position_resource: ResourceAddress,
        position_vault: NonFungibleVault,
        // Admin
        fee_vault: FungibleVault,
    }

    impl YieldMultiplierV1 {
        pub fn instantiate(
            dapp_definition_address: ComponentAddress,
            owner_badge: Proof,
            parent_platform: ComponentAddress,
            supply: ResourceAddress,
            debt: ResourceAddress,
            position_resource: ResourceAddress,
        ) -> Global<YieldMultiplierV1> {
            // Reserve address
            let (address_reservation, component_address) = Runtime::allocate_component_address(YieldMultiplierV1::blueprint_id());

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
                    "name"            => "L3//Cluster - Yield Multiplier v1", locked;
                    "description"     => "Lattic3 cluster component for the 'Yield Multiplier v1' strategy.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            let component_roles = roles! {
                platform => platform_access_rule.clone();
            };

            // Instantisation
            let initial_state = Self {
                owner_badge_address: owner_badge.resource_address(),
                parent_platform,
                supply,
                debt,
                position_resource,
                position_vault: NonFungibleVault::new(position_resource),
                fee_vault: FungibleVault::new(supply),
            };

            let component: Global<YieldMultiplierV1> = initial_state
                .instantiate()
                .prepare_to_globalize(owner_role)
                .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }
    }
}
