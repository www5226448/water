use std::{collections::HashMap, fs};

use starknet::{
    accounts::Call,
    core::types::{self, BlockId, BlockTag, FieldElement, FunctionCall},
    macros::{felt, selector},
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Provider, Url,
    },
};
use tokio::time::{sleep, Duration};

use crate::types::*;
use crate::utils::timestamp;

pub fn decode_pair_data() -> Vec<Pair> {
    // 反序列化JSON数据到Vec<Pair>
    let json_data = fs::read_to_string("./conf/dex.json").expect("Failed to read dex data");
    let pairs: Vec<Pair> = serde_json::from_str(&json_data).expect("JSON was not well-formatted");
    pairs
}

fn construct_pairv2_parameters(pair_info: &Pair) -> Call {
    match pair_info.dex {
        Dex::jedipair => Call {
            to: pair_info.pair_address,
            selector: selector!("get_reserves"),
            calldata: vec![],
        },
        Dex::onepair => Call {
            to: pair_info.pair_address,
            selector: selector!("getReserves"),
            calldata: vec![],
        },
        Dex::myPoolId => Call {
            to: felt!("0x010884171baf1914edc28d7afb619b40a4051cfae78a094a55d230f19e944a28"),
            selector: selector!("get_pool"),
            calldata: vec![pair_info.pair_address],
        },
    }
}

fn encode_calls(calls: &[Call]) -> Vec<FieldElement> {
    let mut execute_calldata: Vec<FieldElement> = vec![calls.len().into()];
    let mut concated_calldata: Vec<FieldElement> = vec![];
    for call in calls.iter() {
        execute_calldata.push(call.to); // to
        execute_calldata.push(call.selector); // selector
        execute_calldata.push(concated_calldata.len().into()); // data_offset
        execute_calldata.push(call.calldata.len().into()); // data_len

        for item in call.calldata.iter() {
            concated_calldata.push(*item);
        }
    }

    execute_calldata.push(concated_calldata.len().into()); // calldata_len
    execute_calldata.extend_from_slice(&concated_calldata);
    execute_calldata
}

pub fn compile_multicall_parameters() -> Vec<Call> {
    let mut calls: Vec<Call> = vec![];
    let pairs = decode_pair_data();
    for p in pairs.iter() {
        let call = construct_pairv2_parameters(p);
        calls.push(call);
    }
    calls
}

pub fn compile_states<'a>(
    pairs: &'a Vec<Pair>,
    raw_states: &'a Vec<FieldElement>,
) -> HashMap<&'a Pair, (FieldElement, FieldElement)> {
    let mut dex_data: HashMap<&Pair, (FieldElement, FieldElement)> = HashMap::new();
    let mut current_index = 2_usize;
    for pair in pairs.iter() {
        match pair.dex {
            Dex::jedipair => {
                let (s0, s1) = (raw_states[current_index + 1], raw_states[current_index + 3]);
                current_index += 6;
                dex_data.insert(pair, (s0, s1));
            }
            Dex::onepair => {
                let (s0, s1) = (raw_states[current_index + 1], raw_states[current_index + 2]);
                current_index += 4;
                dex_data.insert(pair, (s0, s1));
            }
            Dex::myPoolId => {
                let (s0, s1) = (raw_states[current_index + 3], raw_states[current_index + 6]);
                current_index += 11;
                dex_data.insert(pair, (s0, s1));
            }
        }
    }
    dex_data
}

pub async fn retrieve(searcher: &Searcher) -> Vec<FieldElement> {
    let multicall_address =
        felt!("0x05754af3760f3356da99aea5c3ec39ccac7783d925a19666ebbeca58ff0087f4");

    let raw_calls = compile_multicall_parameters();

    loop {
        let raw_call_vectors = encode_calls(&raw_calls);
        let r = searcher
            .0
            .call(
                FunctionCall {
                    contract_address: multicall_address,
                    entry_point_selector: selector!("aggregate"),
                    calldata: raw_call_vectors,
                },
                BlockId::Tag(BlockTag::Latest),
            )
            .await;

        match r {
            Ok(result) => {
                return result;
            }
            _ => {
                sleep(Duration::from_millis(3000)).await;
                continue;
            }
        }
    }
}

pub async fn update_nonce(
    searcher: &Searcher,
    address: FieldElement,
    nonce: FieldElement,
    update_index: u64,
) -> (u64, FieldElement) {
    let mut now = timestamp();
    let delayed = now - update_index;

    if delayed > 300 {
        loop {
            match searcher
                .0
                .get_nonce(BlockId::Tag(BlockTag::Latest), address)
                .await
            {
                Ok(new_nonce) => {
                    now = timestamp();
                    println!("timestamp {:?}, nonce updated {:?}", now, new_nonce);
                    return (now, new_nonce);
                }
                Err(_e) => {
                    sleep(Duration::from_millis(3000)).await;

                    continue;
                }
            }
        }
    } else {
        return (update_index, nonce);
    }
}

pub async fn initialized_nonce(searcher: &Searcher, address: FieldElement) -> (u64, FieldElement) {
    loop {
        match searcher
            .0
            .get_nonce(BlockId::Tag(BlockTag::Latest), address)
            .await
        {
            Ok(n) => {
                let now = timestamp();
                return (now, n);
            }
            Err(_) => {
                sleep(Duration::from_millis(3000)).await;
                continue;
            }
        }
    }
}
