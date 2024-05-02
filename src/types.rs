use serde::{Deserialize, Serialize};
use starknet::core::types::FieldElement;
use std::fs;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Clone)]
pub enum Token {
    USDC,
    USDT,
    DAI,
    ETH,
    WBTC,
    wstETH,
    STRK,
}

impl Token {
    pub fn address(&self) -> FieldElement {
        match self {
            Token::ETH => {
                let addr = "0x049D36570D4e46f48e99674bd3fcc84644DdD6b96F7C741B1562B82f9e004dC7";
                FieldElement::from_hex_be(&addr).unwrap()
            }

            Token::USDC => {
                let addr = "0x053C91253BC9682c04929cA02ED00b3E423f6710D2ee7e0D5EBB06F3eCF368A8";
                FieldElement::from_hex_be(&addr).unwrap()
            }
            Token::USDT => {
                let addr = "0x068F5c6a61780768455de69077E07e89787839bf8166dEcfBf92B645209c0fB8";
                FieldElement::from_hex_be(&addr).unwrap()
            }
            Token::DAI => {
                let addr = "0x00dA114221cb83fa859DBdb4C44bEeaa0BB37C7537ad5ae66Fe5e0efD20E6eB3";
                FieldElement::from_hex_be(&addr).unwrap()
            }
            Token::WBTC => {
                let addr = "0x03Fe2b97C1Fd336E750087D68B9b867997Fd64a2661fF3ca5A7C771641e8e7AC";
                FieldElement::from_hex_be(&addr).unwrap()
            }

            Token::wstETH => {
                let addr = "0x042b8F0484674cA266AC5D08e4aC6A3fE65bd3129795DEF2dCA5c34ecC5F96d2";
                FieldElement::from_hex_be(&addr).unwrap()
            }

            Token::STRK => {
                let addr = "0x04718f5a0Fc34cC1AF16A1cdee98fFB20C31f5cD61D6Ab07201858f4287c938D";
                FieldElement::from_hex_be(&addr).unwrap()
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Clone)]
pub enum Dex {
    jedipair,
    onepair,
    myPoolId,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Hash, Clone)]
pub struct Pair {
    pub dex: Dex,
    pub pair_address: FieldElement,
    pub token0: Token,
    pub token1: Token,
}

use starknet::{
    accounts::{ExecutionEncoding, SingleOwnerAccount},
    core::chain_id,
    providers::{
        jsonrpc::{HttpTransport, JsonRpcClient},
        Url,
    },
    signers::{LocalWallet, SigningKey},
};

pub type MyAccount = SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>;

fn read_keys() -> (FieldElement, FieldElement) {
    let keys = fs::read_to_string("./conf/keys.txt").expect("Failed to read the key file");
    let line: Vec<&str> = keys.split(",").collect();
    let mut line = line.into_iter();
    let address = line.next().expect("Failed to compile address");
    let privateKey = line.next().expect("Failed to compile private key");
    return (
        FieldElement::from_hex_be(address).unwrap(),
        FieldElement::from_hex_be(privateKey).unwrap(),
    );
}

pub fn create_account() -> MyAccount {
    let provider = JsonRpcClient::new(HttpTransport::new(
        Url::parse("https://starknet-mainnet.public.blastapi.io/rpc/v0_7").unwrap(),
    ));
    let (address, private_key) = read_keys();
    let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));

    let account = SingleOwnerAccount::new(
        provider,
        signer,
        address,
        chain_id::MAINNET,
        ExecutionEncoding::New,
    );
    account
}

use once_cell::sync::Lazy;
use std::sync::Arc;

pub struct Fetcher(pub JsonRpcClient<HttpTransport>);

impl Fetcher {
    pub fn new() -> Self {
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse("https://starknet-mainnet.public.blastapi.io/rpc/v0_7").unwrap(),
        ));
        Fetcher(provider)
    }

    pub fn provider(&self) -> &JsonRpcClient<HttpTransport> {
        &self.0
    }
}

pub static FETCHER: Lazy<Arc<Fetcher>> = Lazy::new(|| Arc::new(Fetcher::new()));

pub type Searcher = Arc<Fetcher>;
