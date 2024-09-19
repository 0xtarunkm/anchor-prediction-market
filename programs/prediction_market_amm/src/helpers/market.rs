use super::{CurveError, DepositLiquidityResult};

// Add liquidity
pub fn add_liquidity(
    usdc_amount: u64,
    yes_amount: u64,
    no_amount: u64,
    pool_usdc: u64,
    pool_yes: u64,
    pool_no: u64,
    lp_total_supply: u64,
    precision: u64
) -> Result<DepositLiquidityResult, CurveError> {
    
    let usdc_ratio = (usdc_amount)
        .checked_mul(precision)
        .ok_or(CurveError::Overflow)?
        .checked_div(pool_usdc)
        .ok_or(CurveError::Overflow)?;

    let yes_ratio = (yes_amount)
        .checked_mul(precision)
        .ok_or(CurveError::Overflow)?
        .checked_div(pool_yes)
        .ok_or(CurveError::Overflow)?;

    let no_ratio = (no_amount)
        .checked_mul(precision)
        .ok_or(CurveError::Overflow)?
        .checked_div(pool_no)
        .ok_or(CurveError::Overflow)?;

    let lp_tokens = lp_total_supply
        .checked_mul(usdc_ratio.min(yes_ratio).min(no_ratio) as u64)
        .ok_or(CurveError::Overflow)?
        .checked_div(precision)
        .ok_or(CurveError::Overflow)? as u64;

    Ok(DepositLiquidityResult {
        usdc_amount,
        yes_amount,
        no_amount,
        lp_tokens,
    })
}

