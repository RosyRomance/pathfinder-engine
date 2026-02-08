use alloy::primitives::address;
use pathfinder::{
    finder::{
        types::ContractCandidate,
    },
};

fn main() {
    println!("=================================");
    println!("THIS IS A EXAMPLE");
    let cond = ContractCandidate {
        chain_id: 1,
        contract: address!("0xd0415cf4558A0dBEE8242498D25284476bE3c8f2"),
        deployed_block: 0,
        receives_token: None,
        has_balance: None,
        verify_attempts: 0,
        first_receive_block: None,
        abi: None,
    };
    println!("contract: {:?}", cond);
    println!("Score: 80.0");
    println!("APR: **% ");

    println!("APR Risk: * * ");
    println!("TVL Concentration Risk: * * ");
    println!("Owner Privilege Risk: * * ");
    println!("Proxy Admin EOA Risk: * * ");
    println!("Reward Inflation Risk: * * ");
    println!("TVL Volatility Risk: * * ");
    println!("Very New Contract Risk: * * ");

    println!("=================================");

}