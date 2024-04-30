use crate::types::*;
use starknet::core::types::FieldElement;
use starknet::macros::felt;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn whether_profitable(
    amount_in: &FieldElement,
    back: &FieldElement,
    threshold: &FieldElement,
) -> bool {
    if back > amount_in && (*back - *amount_in > *threshold) {
        return true;
    }
    false
}

fn get_amount_out(
    reserve0: &FieldElement,
    reserve1: &FieldElement,
    amountIn: &FieldElement,
) -> FieldElement {
    let amountInWithFee = *amountIn * felt!("997"); // amountInWithFee = amountIn.mul(997);
                                                    // numerator = amountInWithFee.mul(reserveOut);
    let numerator = amountInWithFee * (*reserve1);
    let denominator = *reserve0 * felt!("1000") + amountInWithFee;
    // denominator = reserveIn.mul(1000).add(amountInWithFee);
    let amountOut = numerator.floor_div(denominator);
    //println!("{:?}{:?}{:?}{:?}", amountIn, reserve0, reserve1, amountOut);
    amountOut
}

pub fn uniswapv2_getAmountOut(
    token0: &Token,
    _token1: &Token,
    reserve0: &FieldElement,
    reserve1: &FieldElement,
    amountIn: &FieldElement,
    tokenIn: &Token,
) -> FieldElement {
    if token0 != tokenIn {
        return get_amount_out(reserve1, reserve0, amountIn);
    }
    get_amount_out(reserve0, reserve1, amountIn)
}

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[test]
fn test_field() {
    let r0 = felt!("17000000000000000000");
    let r1 = felt!("59499999999999996854272");
    let amountin = felt!("500000000000000000");
    assert_eq!(
        get_amount_out(&r0, &r1, &amountin),
        felt!("1695045289596251017621")
    )
}

#[test]
fn test_profitable() {
    let amount_in = felt!("500000000000000000");
    let back = felt!("472874785449689274");
    let threshold = felt!("333333333333333");
    assert_eq!(whether_profitable(&amount_in, &back, &threshold), false);
}
