use std::collections::HashMap;

use crate::types::*;
use crate::utils::{timestamp, uniswapv2_getAmountOut, whether_profitable};
use log::{info, warn};
use starknet::accounts::AccountError;
use starknet::accounts::AccountError::Provider;
use starknet::accounts::{Account, Call, RawExecution};
use starknet::core::types::{FieldElement, StarknetError};
use starknet::macros::{felt, selector};
use starknet::providers::ProviderError;

pub fn task2(token0: Token, token1: Token, pairs: &Vec<Pair>) -> Vec<Pair> {
    let mut v: Vec<Pair> = vec![];
    for p in pairs.iter() {
        if p.token0 == token0 && p.token1 == token1 {
            v.push(p.clone());
        }
        if p.token1 == token0 && p.token0 == token1 {
            v.push(p.clone());
        }
    }
    return v;
}

fn check_medium(token_in: &Token, token0: &Token, token1: &Token) -> Token {
    let medium = if token_in == token0 {
        token1.clone()
    } else {
        token0.clone()
    };
    medium
}

pub async fn bridge2(
    account: &MyAccount,
    nonce: FieldElement,
    amount_in: &FieldElement,
    token_in: Token,
    paths: Vec<Pair>,
    dex_states: &HashMap<&Pair, (FieldElement, FieldElement)>,
) {
    let threshold = felt!("15625000000000");
    let n = paths.len();
    for i in 0..n {
        for j in i + 1..n {
            let p0 = paths[i].clone();
            let p1 = paths[j].clone();
            let (r0, r1) = dex_states.get(&p0).unwrap();
            let mut medium =
                uniswapv2_getAmountOut(&p0.token0, &p0.token1, r0, r1, amount_in, &token_in);
            let token_medium = check_medium(&token_in, &p0.token0, &p0.token1);
            let (r0, r1) = dex_states.get(&p1).unwrap();
            let back =
                uniswapv2_getAmountOut(&p1.token0, &p1.token1, r0, r1, &mut medium, &token_medium);

            if whether_profitable(amount_in, &back, &threshold) {
                info!(
                    "Find an opportunity, 
                tokenin {:?} amoutin {:?} token_medium {:?} medium{:?},back {:?}",
                    token_in, amount_in, token_medium, medium, back
                );
                executed_tx(
                    account,
                    nonce,
                    &token_in,
                    amount_in,
                    &token_medium,
                    &medium,
                    &token_in,
                    &back,
                    p0,
                    p1,
                )
                .await;
            }
            //reserve the path
            let p1 = paths[i].clone();
            let p0 = paths[j].clone();
            let (r0, r1) = dex_states.get(&p0).unwrap();
            let mut medium =
                uniswapv2_getAmountOut(&p0.token0, &p0.token1, r0, r1, amount_in, &token_in);
            let token_medium = check_medium(&token_in, &p0.token0, &p0.token1);
            let (r0, r1) = dex_states.get(&p1).unwrap();
            let back =
                uniswapv2_getAmountOut(&p1.token0, &p1.token1, r0, r1, &mut medium, &token_medium);

            if whether_profitable(amount_in, &back, &threshold) {
                info!(
                    "Find an opportunity, 
                tokenin {:?} amoutin {:?} token_medium {:?} medium{:?},back {:?}",
                    token_in, amount_in, token_medium, medium, back
                );
                executed_tx(
                    account,
                    nonce,
                    &token_in,
                    amount_in,
                    &token_medium,
                    &medium,
                    &token_in,
                    &back,
                    p0,
                    p1,
                )
                .await;
            }
        }
    }
}

fn mapping_contract(dex: &Dex) -> FieldElement {
    match dex {
        Dex::jedipair => FieldElement::from_hex_be(
            "0x041fd22b238fa21cfcf5dd45a8548974d8263b3a531a60388411c5e230f97023",
        )
        .unwrap(),
        Dex::onepair => FieldElement::from_hex_be(
            "0x07a6f98c03379b9513ca84cca1373ff452a7462a3b61598f0af5bb27ad7f76d1",
        )
        .unwrap(),
        Dex::myPoolId => FieldElement::from_hex_be(
            "0x010884171baf1914edc28d7afb619b40a4051cfae78a094a55d230f19e944a28",
        )
        .unwrap(),
    }
}
fn mapping_selector(dex: &Dex) -> FieldElement {
    match dex {
        Dex::jedipair => selector!("swap_exact_tokens_for_tokens"),
        Dex::onepair => selector!("swapExactTokensForTokens"),
        Dex::myPoolId => selector!("swap"),
    }
}

fn construct_calldata(
    address: &FieldElement,
    token_in: &Token,
    amount_in: &FieldElement,
    token_out: &Token,
    amount_out: &FieldElement,
    dex: &Dex,
    pair: &Pair,
    deadline: &FieldElement,
) -> Call {
    match dex {
        Dex::myPoolId => Call {
            to: mapping_contract(dex),
            selector: mapping_selector(dex),
            calldata: vec![
                pair.pair_address,
                token_in.address(),
                *amount_in,
                felt!("0"),
                *amount_out,
                felt!("0"),
            ],
        },
        _ => Call {
            to: mapping_contract(dex),
            selector: mapping_selector(dex),
            calldata: vec![
                *amount_in,
                felt!("0"),
                *amount_out,
                felt!("0"),
                felt!("2"),
                token_in.address(),
                token_out.address(),
                *address,
                *deadline,
            ],
        },
    }
}

async fn executed_tx(
    account: &MyAccount,
    nonce: FieldElement,
    token_in: &Token,
    amount_in: &FieldElement,
    token_medium: &Token,
    medium: &FieldElement,
    token_back: &Token,
    back: &FieldElement,
    p0: Pair,
    p1: Pair,
) {
    let address = account.address();

    let now = timestamp() / 1800 * 1800 + 3000;
    let deadline = FieldElement::from_dec_str(&now.to_string()).unwrap();

    let call0 = construct_calldata(
        &address,
        token_in,
        amount_in,
        &token_medium,
        &medium,
        &p0.dex,
        &p0,
        &deadline,
    );
    let call1 = construct_calldata(
        &address,
        token_medium,
        medium,
        token_back,
        back,
        &p1.dex,
        &p1,
        &deadline,
    );
    assert!(back > amount_in);
    let max_fee: FieldElement = *back - *amount_in;
    let this_execution: starknet::accounts::Execution<
        '_,
        starknet::accounts::SingleOwnerAccount<
            starknet::providers::JsonRpcClient<starknet::providers::jsonrpc::HttpTransport>,
            starknet::signers::LocalWallet,
        >,
    > = account
        .execute(vec![call0, call1])
        .nonce(nonce)
        .max_fee(max_fee);

    let prepared_execution = this_execution.prepared();

    if let Ok(execution) = prepared_execution {
        let tx_hash = execution.transaction_hash(false);

        let r_tx = Tx_pool.read().unwrap();
        if r_tx.contains(&tx_hash) {
            println!("the transaction was broadcasted");
        } else {
            let mut w_tx = Tx_pool.write().unwrap();
            w_tx.push(tx_hash);
            let tx = execution.send().await;
            println!("executed a new tx {:?}", tx);
        }
    }
}

use std::sync::RwLock;

static Tx_pool: RwLock<Vec<FieldElement>> = RwLock::new(Vec::new());
