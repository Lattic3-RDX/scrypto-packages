use scrypto_test::prelude::*;
use std::path::Path;

//] ------------------ General ----------------- */
pub type Ledger = LedgerSimulator<NoExtension, InMemorySubstateDatabase>;

pub fn merge_path(input_path: &str) -> String {
    let path = Path::new(this_package!()).join(input_path).canonicalize().unwrap();
    let path_string = path.to_str().unwrap();

    path_string.to_string()
}

//] ------------------ Account ----------------- */
#[derive(Debug, Clone, Copy)]
pub struct SimAccount {
    pub address: ComponentAddress,
    pub public_key: Secp256k1PublicKey,
    // pub private_key: Secp256k1PrivateKey,
}

impl SimAccount {
    pub fn new(ledger: &mut Ledger) -> Self {
        let (public_key, _private_key, address) = ledger.new_allocated_account();

        Self { public_key, address }
    }

    pub fn global_id(&self) -> NonFungibleGlobalId {
        NonFungibleGlobalId::from_public_key(&self.public_key)
    }
}
