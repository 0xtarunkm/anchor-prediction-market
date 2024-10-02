#[macro_export]
macro_rules! assert_non_zero {
    ($array:expr) => {
        if $array.contains(&0u64) {
            return err!(crate::error::MarketError::ZeroBalance)
        }
    };
}

#[macro_export]
macro_rules! assert_not_locked {
    ($lock:expr) => {
        if $lock == true {
            return err!(crate::error::MarketError::PoolLocked)
        }
    };
}

#[macro_export]
macro_rules! assert_not_expired {
    ($expiration:expr) => {
        if Clock::get()?.unix_timestamp > $expiration {
            return err!(crate::error::MarketError::OfferExpired);
        }
    };
}