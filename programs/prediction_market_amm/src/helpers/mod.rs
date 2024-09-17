#[macro_export]
macro_rules! assert_non_zero {
    ($array:expr) => {
        if $array.contains(&0u64) {
            return err!(MarketError::ZeroBalance)
        }
    };
}

#[macro_export]
macro_rules! assert_not_locked {
    ($lock:expr) => {
        if $lock == true {
            return err!(MarketError::PoolLocked)
        }
    };
}

#[macro_export]
macro_rules! assert_not_expired {
    ($expiration:expr) => {
        if Clock::get()?.unix_timestamp > $expiration {
            return err!(MarketError::OfferExpired);
        }
    };
}

#[macro_export]
macro_rules! has_update_authority {
    ($x:expr) => {
        match $x.market.authority {
            Some(a) => {
                require_keys_eq!(a, $x.admin.key(), crate::error::MarketError::InvalidAuthority);
            },
            None => return err!(crate::error::MarketError::NoAuthoritySet)
        }
    };
}
