pub fn calculate_output(input_amount: u64, input_reserve: u64, output_reserve: u64) -> u64 {
    let input_amount_with_fee = input_amount * 997;
    let numerator = input_amount_with_fee * output_reserve;
    let denominator = (input_reserve * 1000) + input_amount_with_fee;
    let result = (numerator / denominator) as u64;
    result
}