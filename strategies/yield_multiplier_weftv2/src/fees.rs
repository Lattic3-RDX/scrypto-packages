/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* -------------- Fee Breakpoints ------------- */
#[derive(ScryptoSbor, Debug, Clone, Copy)]
/// Indicates fee amounts (in XRD) for each operation.
pub struct FeeStructure {
    pub open: Decimal,
    pub close: Decimal,
    pub execute: Decimal,
}

impl FeeStructure {
    pub fn default() -> Self {
        Self { open: dec!(16), close: dec!(4), execute: dec!(4) }
    }

    pub fn set(&mut self, open: Option<Decimal>, close: Option<Decimal>, execute: Option<Decimal>) {
        self.open = open.unwrap_or(self.open);
        self.close = close.unwrap_or(self.close);
        self.execute = execute.unwrap_or(self.execute);
    }
}
