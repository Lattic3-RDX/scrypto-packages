use scrypto_test::prelude::*;
use std::path::Path;

pub type Ledger = LedgerSimulator<NoExtension, InMemorySubstateDatabase>;

pub fn merge_path(input_path: &str) -> String {
    let path = Path::new(this_package!()).join(input_path).canonicalize().unwrap();
    let path_string = path.to_str().unwrap();

    path_string.to_string()
}
