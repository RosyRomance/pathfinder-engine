use anyhow::Result;
use clickhouse::Client;
use std::sync::Arc;
use alloy::primitives::address;
use futures::TryFutureExt;
use pathfinder::{
    finder::{
        self, alloy_evm::AlloyEvmClient, config::ScannerConfig, discovery::SimpleDiscovery, evm::EvmClient, filter::BehaviorFilter, inspect::ShallowInspector, scanner::Scanner, types::{BlockRange, ContractCandidate}
    },
    risk::{
        self, 
    },
    sink::{
        client::ClickhouseClient,
        project_store::{ClickhouseStore, ProjectStore},
        risk_store::ClickHouseRiskStore,
    },
};

// ===================== Demo main =====================

#[tokio::main]
async fn main() -> Result<()> {
    // let rpc = "https://mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    // let rpc = "https://arbitrum-mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    // let rpc = "https://eth-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";
    // let rpc = "https://arb-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";
    let rpc = "https://rpc.ankr.com/eth/55ca74443b1e46c9e322c05381284b4b6ac6ab1c31e722a7e38ae503de1ae195";

    let evm_client = AlloyEvmClient::new_client(rpc)?;
    let client = Client::default()
        .with_url("http://127.0.0.1:8123")
        .with_user("default")
        .with_password("StrongerPwd456!")
        .with_database("pathfinder");
    let ch_client = ClickhouseClient { client: client.clone() };
        
    // 1) ----- 
    // To simplify the model, the staking project currently only accepts ERC20 tokens. However, during code debugging, we were unable to find a suitable real-world project that accepts ERC20 tokens.
    // Therefore, for demonstration purposes, an example staking project is included here.
    insert_example(&ch_client).await?;

    // 2) ----- scan to find staking candidates and verify them
    let finder_store = Arc::new(ClickhouseStore::new(&ch_client));
    let scanner = finder::build::build(finder_store.clone(), evm_client)?;
    let latest_block = 24383912;  
    let block_range = BlockRange {
        from: latest_block,
        to: latest_block + 5,
    };  

    let now = chrono::Utc::now();
    let snapshot_date = now.format("%Y-%m-%d").to_string();
    let now_unix = now.timestamp() as u64;
    match scanner.run_once(block_range, &snapshot_date, now_unix).await {
        Ok(inserted) => println!("Scan complete. New projects inserted: {}", inserted),
        Err(e) => eprintln!("Scan failed with error: {:?}", e),
    }

    // ----- 3) scan the risk item of verified contracts
    let risk_store = ClickHouseRiskStore { client: client.clone() };
    let snapshots = finder_store.snapshots_list().await?;
    let verified = finder_store.verified_list().await?;
    let risk_engine = risk::build::build(&snapshots, &verified);
    // match risk_engine.run().await {
    //     Ok(RiskReport) => println!("ok"),
    //     Err(e) => eprintln!("error: {:?}", e),
    // }

    Ok(())
}


async fn insert_example(client: &ClickhouseClient) -> Result<()> {
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

    // let ch_client = ClickhouseClient { 
    //     client:  Client::default()
    //         .with_url("http://127.0.0.1:8123")
    //         .with_user("default")
    //         .with_password("StrongerPwd456!")
    //         .with_database("pathfinder")
    //     };

    client.truncate_table("pending_contracts").await?;
    client.insert_pending_contracts(1, &[cond]).await?;

    Ok(())
}
