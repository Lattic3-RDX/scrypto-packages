// Libraries
use scrypto::prelude::*;

/* ------------------- Time ------------------- */
pub const SECONDS_PER_YEAR: i64 = 60 * 60 * 24 * 365;

pub fn now() -> i64 {
    Clock::current_time(TimePrecisionV2::Second).seconds_since_unix_epoch
}
