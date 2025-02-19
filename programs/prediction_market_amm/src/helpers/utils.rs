use anchor_lang::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use rust_decimal::prelude::*;

use crate::error::MarketError;

pub const PRECISION: u32 = 6;
pub const DEFAULT_B: u64 = 1_000_000_000;

#[derive(Debug)]
pub struct LMSRCalculator {
    pub b: Decimal,
    pub yes_shares: Decimal,
    pub no_shares: Decimal,
}

impl LMSRCalculator {
    pub fn new(b: u64, yes_shares: u64, no_shares: u64) -> Self {
        Self {
            b: Decimal::from(b),
            yes_shares: Decimal::from(yes_shares),
            no_shares: Decimal::from(no_shares),
        }
    }

    pub fn calculate_cost_to_buy(&self, shares: u64, is_yes: bool) -> Result<u64> {
        let current_cost = self.calculate_cost()?;
        let shares_decimal = Decimal::from(shares);
        
        let new_yes_shares = if is_yes {
            self.yes_shares + shares_decimal
        } else {
            self.yes_shares
        };
        
        let new_no_shares = if !is_yes {
            self.no_shares + shares_decimal
        } else {
            self.no_shares
        };

        let new_cost = Self {
            b: self.b,
            yes_shares: new_yes_shares,
            no_shares: new_no_shares,
        }.calculate_cost()?;

        let cost_difference = new_cost - current_cost;
        Ok(cost_difference.round_dp(PRECISION).to_u64().ok_or(MarketError::MathOverflow)?)
    }

    pub fn calculate_cost(&self) -> Result<Decimal> {
        let yes_term = self.exp(self.yes_shares / self.b)?;
        let no_term = self.exp(self.no_shares / self.b)?;
        
        let sum = yes_term + no_term;
        let result = self.ln(sum)?;
        
        Ok(result * self.b)
    }

    pub fn calculate_price(&self, is_yes: bool) -> Result<Decimal> {
        let yes_term = self.exp(self.yes_shares / self.b)?;
        let no_term = self.exp(self.no_shares / self.b)?;
        
        let denominator = yes_term + no_term;
        let numerator = if is_yes { yes_term } else { no_term };
        
        Ok(numerator / denominator)
    }

    fn exp(&self, x: Decimal) -> Result<Decimal> {
        let mut sum = dec!(1.0);
        let mut term = dec!(1.0);
        
        for i in 1..=10 {
            term = term * x / Decimal::from(i);
            sum += term;
        }
        
        Ok(sum)
    }

    fn ln(&self, x: Decimal) -> Result<Decimal> {
        if x <= dec!(0.0) {
            return Err(MarketError::MathOverflow.into());
        }

        let mut guess = dec!(1.0);
        for _ in 0..10 {
            guess = guess + dec!(2.0) * (x - self.exp(guess)?) / (x + self.exp(guess)?);
        }
        
        Ok(guess)
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
    let fees = Decimal::from(input_amount) * dec!(0.01);
    let input_after_fees = Decimal::from(input_amount) - fees;

    if is_buying {
        let price = calculator.calculate_price(is_yes)?;
        Ok((input_after_fees / price).round_dp(0).to_u64().ok_or(MarketError::MathOverflow)?)
    } else {
        calculator.calculate_cost_to_buy(input_after_fees.to_u64().unwrap(), is_yes)
    }
}
