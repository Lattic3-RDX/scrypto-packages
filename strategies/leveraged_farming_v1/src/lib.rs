use scrypto::prelude::*;

#[blueprint]
mod leveraged_farming_v1_factory {
    //] --------------- Scrypto Setup -------------- /

    //] ------------- Factory Blueprint ------------- /

    struct LeveragedFarmingV1Factory {}

    impl LeveragedFarmingV1Factory {
        pub fn instantiate(dapp_definition_address: ComponentAddress, owner_badge: Proof) -> Global<LeveragedFarmingV1Factory> {
            // Reserve address
            let (address_reservation, component_address) = Runtime::allocate_component_address(LeveragedFarmingV1Factory::blueprint_id());

            //] Authorisation
            // Component
            let component_access_rule: AccessRule = rule!(require(global_caller(component_address)));

            // Component owner
            // - let owner_badge: Bucket = ResourceBuilder::new_fungible(OwnerRole::None)
            // -     .divisibility(DIVISIBILITY_NONE)
            // -     .metadata(metadata! {init {
            // -         "name"        => "Lattic3 Owner Badge", locked;
            // -         "description" => "Badge representing the owner of the Lattic3 lending platform", locked;
            // -     }})
            // -     .mint_initial_supply(1)
            // -     .into();
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
                    "name"            => "L3//Factory - Leveraged Yield Farming v0.1", locked;
                    "description"     => "Lattic3 factory component for the 'Leveraged Yield Farming V1' strategy.", locked;
                    "dapp_definition" => dapp_definition_address, updatable;
                }
            };

            // Roles
            // let component_roles = roles! {};

            // Instantisation
            let component: Global<LeveragedFarmingV1Factory> = Self {}
                .instantiate()
                .prepare_to_globalize(owner_role)
                // .roles(component_roles)
                .metadata(component_metadata)
                .with_address(address_reservation)
                .globalize();

            component
        }
    }
}
