use std::{error::Error, fmt};

#[derive(Debug, Clone, Copy)]
pub enum OutcomeToken {
    YES,
    NO,
}

#[derive(Debug)]
pub struct SpotPrice {
    pub amount: u128,
    pub precision: u32,
}

#[derive(Debug)]
pub struct SwapResult {
    pub usdc_amount: u64,
    pub token_amount: u64,
    pub fee: u64,
    pub token_price: f64,
}

#[derive(Debug)]
pub struct WithdrawLiquidityResult {
    pub usdc_amount: u64,
    pub yes_amount: u64,
    pub no_amount: u64,
    pub lp_tokens: u64,
}

#[derive(Debug)]
pub struct DepositLiquidityResult {
    pub usdc_amount: u64,
    pub yes_amount: u64,
    pub no_amount: u64,
    pub lp_tokens: u64,
}

#[derive(Debug)]
pub struct SettleResult {
    pub total_payout: u64,
}

#[derive(Debug)]
pub enum CurveError {
    InvalidPrecision,
    Overflow,
    Underflow,
    InvalidFeeAmount,
    InsufficientBalance,
    ZeroBalance,
    SlipageLimitExceeded,
    MarketAlreadySettled,
    MarketNotSettled,
    Unauthorized,
}

impl Error for CurveError {}

impl fmt::Display for CurveError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn calculate_output(input_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    let input_amount_with_fee = input_amount * 997;
    let numerator = input_amount_with_fee * output_reserve;
    let denominator = (input_reserve * 1000) + input_amount_with_fee;
    let result = (numerator / denominator) as u64;
    result
}