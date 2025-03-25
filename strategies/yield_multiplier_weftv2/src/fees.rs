/* ------------------ Imports ----------------- */
use scrypto::prelude::*;

/* -------------- Fee Breakpoints ------------- */
// Points are (liquidity, annual fee rate)
#[derive(ScryptoSbor, Debug, Clone)]
pub struct FeePoints {
    pub p0: (Decimal, Decimal),
    pub p1: (Decimal, Decimal),
    pub p2: (Decimal, Decimal),
}

impl FeePoints {
    pub fn new() -> Self {
        let p0 = (dec!(0), dec!(0.1));
        // liq: 0 -> 1_000 ; fee: 0.1 -> 0.08
        let p1 = (dec!(1_000), dec!(0.08));
        // liq: 1_000 -> 10_000 ; fee: 0.08 -> 0.05
        let p2 = (dec!(10_000), dec!(0.05));
        // liq: 10_000 -> _ ; fee: 0.05

        let points = [p0, p1, p2];

        // Validate points
        for i in 0..points.len() - 1 {
            assert!(points[i].0 < points[i + 1].0, "Fee breakpoints must be in order");
        }

        Self { p0, p1, p2 }
    }

    pub fn set_fee_points(&mut self, p0: Option<(Decimal, Decimal)>, p1: Option<(Decimal, Decimal)>, p2: Option<(Decimal, Decimal)>) {
        let p0 = p0.unwrap_or(self.p0);
        let p1 = p1.unwrap_or(self.p1);
        let p2 = p2.unwrap_or(self.p2);

        let points = [p0, p1, p2];

        // Validate points
        for i in 0..points.len() - 1 {
            assert!(points[i].0 < points[i + 1].0, "Fee breakpoints must be in order");
        }

        // Assign points
        self.p0 = p0;
        self.p1 = p1;
        self.p2 = p2;
    }

    pub fn get_fee_rate(&self, liquidity: Decimal) -> Decimal {
        let points = [self.p0, self.p1, self.p2];

        // Do not collect fees for liquidity <= 0
        if liquidity <= dec!(0) {
            return dec!(0);
        }

        // Find fee rate for liquidity
        for i in 0..points.len() - 1 {
            let (l0, f0) = points[i];
            let (l1, f1) = points[if i + 1 >= points.len() { i } else { i + 1 }];

            if liquidity <= l1 {
                // Skip math if l0 == l1; assume last point is infinite
                if (l1 - l0) == dec!(0) {
                    return f0;
                }

                // g = (f1 - f0) / (l1 - l0)
                let g = (f1.checked_sub(f0).unwrap()).checked_div(l1.checked_sub(l0).unwrap()).unwrap();
                // f0 + g * (liquidity - l0)
                return f0.checked_add(g.checked_mul(liquidity.checked_sub(l0).unwrap()).unwrap()).unwrap();
            }
        }

        panic!("Unable to find fee rate for liquidity {:?}", liquidity);
    }
}
