use anchor_lang::error_code;
use prediction_market_curve::CurveError;

#[error_code]
pub enum MarketError {
    #[msg("fee percentage can only be between 0 and 100")]
    FeePercentErr,
    #[msg("DefaultError")]
    DefaultError,
    #[msg("Offer expired")]
    OfferExpired,
    #[msg("This pool is locked")]
    PoolLocked,
    #[msg("Slippage exceeded")]
    SlippageExceeded,
    #[msg("Overflow detected")]
    Overflow,
    #[msg("Underflow detected")]
    Underflow,
    #[msg("Invalid Token")]
    InvalidToken,
    #[msg("No liquidity in pool")]
    NoLiquidityInPool,
    #[msg("Bump error.")]
    BumpError,
    #[msg("Curve error.")]
    CurveError,
    #[msg("Fee is greater than 100%")]
    InvalidFee,
    #[msg("Invalid update authority")]
    InvalidAuthority,
    #[msg("No update authority set.")]
    NoAuthoritySet,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Invalid precision")]
    InvalidPrecision,
    #[msg("Insufficient balance.")]
    InsufficientBalance,
    #[msg("Zero balance.")]
    ZeroBalance,
    #[msg("Market already settled")]
    MarketAlreadySettled,
    #[msg("Market not settled")]
    MarketNotSettled,
    #[msg("Not authorized to perform this")]
    Unauthorized
}

impl From<CurveError> for MarketError {
    fn from(error: CurveError) -> MarketError {
        match error {
            CurveError::InvalidPrecision => MarketError::InvalidPrecision,
            CurveError::Overflow => MarketError::Overflow,
            CurveError::Underflow => MarketError::Underflow,
            CurveError::InvalidFeeAmount => MarketError::InvalidFee,
            CurveError::InsufficientBalance => MarketError::InsufficientBalance,
            CurveError::ZeroBalance => MarketError::ZeroBalance,
            CurveError::SlipageLimitExceeded => MarketError::SlippageExceeded,
            CurveError::MarketAlreadySettled => MarketError::MarketAlreadySettled,
            CurveError::MarketNotSettled => MarketError::MarketNotSettled,
            CurveError::Unauthorized => MarketError::Unauthorized
        }
    }
}