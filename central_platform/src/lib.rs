/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* ------------ Platform Blueprint ------------ */
#[blueprint]
mod central_platform {
    struct Platform {}

    impl Platform {
        pub fn instantiate() -> Global<Platform> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
