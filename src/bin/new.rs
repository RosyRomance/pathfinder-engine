use anyhow::Result;
use clickhouse::Client;
use std::sync::Arc;
use pathfinder::{
    finder::{
        self, 
        alloy_evm::AlloyEvmClient, 
        types::BlockRange
    }, 
    sink::{
        client::ClickhouseClient, project_store::ClickhouseStore, 
        risk_store::ClickHouseRiskStore,
    },
};

// ===================== Demo main =====================

#[tokio::main]
async fn main() -> Result<()> {
    // let rpc = "https://mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    // let rpc = "https://arbitrum-mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    let rpc = "https://eth-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";
    // let rpc = "https://arb-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";

    let evm_client = AlloyEvmClient::new_client(rpc)?;

    let ch_client = ClickhouseClient { client:  Client::default().with_url("http://127.0.0.1:8123").with_database("pathfinder")};
    let store = Arc::new(ClickhouseStore::new(&ch_client));

    let scanner = finder::build::build(store, evm_client).expect("build Scanner failed");

    let latest_block = 19499213u64;    // 原生质押
    // let latest_block = 19584321u64;  // Lido
    let block_range = BlockRange {
        from: latest_block,
        to: latest_block + 5000,
    };  
    let snapshot_date = "2024-06-01";
    let now_unix = chrono::Utc::now().timestamp() as u64;
    match scanner.run_once(block_range, snapshot_date, now_unix).await {
        Ok(inserted) => println!("Scan complete. New projects inserted: {}", inserted),
        Err(e) => eprintln!("Scan failed with error: {:?}", e),
    }
    Ok(())
}