use anchor_lang::prelude::msg;

pub fn calculate_output(input_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    // 100_000_000
    let fees_amount = input_amount * 100 / 10_000;
    let input_amount_with_fee = input_amount - fees_amount;
    let numerator = input_amount_with_fee * output_reserve;
    let denominator = (input_reserve) + input_amount_with_fee;
    let result = (numerator / denominator) as u64;

    msg!("result {}", result);
    msg!("numerator {}", numerator);
    msg!("denominator {}", denominator);
    result
}
