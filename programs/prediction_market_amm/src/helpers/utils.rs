use anchor_lang::prelude::*;

use crate::error::MarketError;

pub const PRECISION: u64 = 1_000_000;
pub const DEFAULT_B: u64 = 1_000_000_000;

#[derive(Debug)]
pub struct LMSRCalculator {
    pub b: u64,
    pub yes_shares: u64,
    pub no_shares: u64,
}

impl LMSRCalculator {
    pub fn new(b: u64, yes_shares: u64, no_shares: u64) -> Self {
        Self {
            b,
            yes_shares,
            no_shares,
        }
    }

    pub fn calculate_cost_to_buy(&self, shares: u64, is_yes: bool) -> Result<u64> {
        let current_cost = self.calculate_cost()?;
        
        let new_yes_shares = if is_yes {
            self.yes_shares.checked_add(shares).ok_or(MarketError::MathOverflow)?
        } else {
            self.yes_shares
        };
        
        let new_no_shares = if !is_yes {
            self.no_shares.checked_add(shares).ok_or(MarketError::MathOverflow)?
        } else {
            self.no_shares
        };

        let new_cost = Self::new(self.b, new_yes_shares, new_no_shares).calculate_cost()?;
        new_cost.checked_sub(current_cost).ok_or(MarketError::MathOverflow.into())
    }

    pub fn calculate_cost(&self) -> Result<u64> {
        let yes_term = self.fixed_point_exp(self.yes_shares, self.b)?;
        let no_term = self.fixed_point_exp(self.no_shares, self.b)?;
        
        let sum = yes_term.checked_add(no_term).ok_or(MarketError::MathOverflow)?;
        let result = self.fixed_point_ln(sum)?;
        
        Ok(result.checked_mul(self.b).ok_or(MarketError::MathOverflow)? / PRECISION)
    }

    pub fn calculate_price(&self, is_yes: bool) -> Result<u64> {
        let yes_term = self.fixed_point_exp(self.yes_shares, self.b)?;
        let no_term = self.fixed_point_exp(self.no_shares, self.b)?;
        
        let denominator = yes_term.checked_add(no_term).ok_or(MarketError::MathOverflow)?;
        let numerator = if is_yes { yes_term } else { no_term };
        
        Ok(numerator.checked_mul(PRECISION).ok_or(MarketError::MathOverflow)? / denominator)
    }

    fn fixed_point_exp(&self, x: u64, b: u64) -> Result<u64> {
        let scaled_x = x.checked_mul(PRECISION).ok_or(MarketError::MathOverflow)? / b;
        Ok(scaled_x.checked_add(PRECISION).ok_or(MarketError::MathOverflow)?)
    }

    fn fixed_point_ln(&self, x: u64) -> Result<u64> {
        Ok(x.checked_mul(PRECISION).ok_or(MarketError::MathOverflow)? / PRECISION)
    }
}

pub fn calculate_lmsr_output(
    input_amount: u64,
    yes_shares: u64,
    no_shares: u64,
    is_buying: bool,
    is_yes: bool,
) -> Result<u64> {
    let calculator = LMSRCalculator::new(DEFAULT_B, yes_shares, no_shares);
    
    // Apply fees (1%)
    let fees = input_amount.checked_mul(100).ok_or(MarketError::MathOverflow)? / 10_000;
    let input_after_fees = input_amount.checked_sub(fees).ok_or(MarketError::MathOverflow)?;

    if is_buying {
        let price = calculator.calculate_price(is_yes)?;
        Ok(input_after_fees.checked_mul(PRECISION).ok_or(MarketError::MathOverflow)? / price)
    } else {
        calculator.calculate_cost_to_buy(input_after_fees, is_yes)
    }
}
