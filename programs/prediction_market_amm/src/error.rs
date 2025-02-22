use anchor_lang::error_code;

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
    Unauthorized,
    #[msg("Market is not expired")]
    MarketNotEnded,
    #[msg("Math overflow")]
    MathOverflow,
    #[msg("Math underflow")]
    MathUnderflow,
    #[msg("Invalid shares")]
    InvalidShares,
    #[msg("Invalid price")]
    InvalidPrice,
    #[msg("Invalid cost")]
    InvalidCost,
}
