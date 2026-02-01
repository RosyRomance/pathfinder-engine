use tokio::sync::mpsc;
use anyhow::Result;
use std::{collections::HashMap, iter::Scan};
use pathfinder::new_project_scanner::{
    alloy_evm::{AlloyEvmClient, SharedProvider}, 
    inspect::ShallowInspector,
    discovery::SimpleDiscovery, 
    config::ScannerConfig,
    filter::BehaviorFilter,
    scanner::Scanner,
    store::MemoryStore,
    types::{BlockNumber, BlockRange},
    errors::ScanError,
};

// ===================== Demo main =====================

#[tokio::main]
async fn main() -> Result<()> {
    // let rpc = "https://mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    // let rpc = "https://arbitrum-mainnet.infura.io/v3/0ef6610d981448148aaa9f2d9c767e8d";
    let rpc = "https://eth-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";
    // let rpc = "https://arb-mainnet.g.alchemy.com/v2/vrWxm65DXY22173CY6Zi28BEwiEcxnwU";

    let config = ScannerConfig {
        chain_id: 1,
        min_confidence: 0.75,
        lookback_blocks: 1000,
        max_candidates_per_run: 100,
        retain_balance_min_blocks: 10,  
        stake_selectors_hex: vec![
            "a694fc3a".to_string(), // deposit()
            "4e71d92d".to_string(), // stake(uint256)
            "f8b2cb4f".to_string(), // stakeFor(address,uint256)
            "2e1a7d4d".to_string(), // withdraw(uint256)
            "f305d719".to_string(), // addLiquidity(...)
            "8803dbee".to_string(), // mint(uint256)
        ],
    };

    let client = AlloyEvmClient::new_client(rpc)?;
    let discovery = SimpleDiscovery {
        chain_id: config.chain_id,
        client: client.clone(),
    }; 
    let filter = BehaviorFilter {
        cfg: config.clone(),
        client: client.clone(),
    };
    let inspector = ShallowInspector {
        cfg: config.clone(),
        client: client.clone(),
    };
    let store = MemoryStore::new();

    let scanner = Scanner {
        cfg: config,
        discovery,
        filter,
        inspector,
        store,
    };

    let latest_block = 19499213u64;    // 原生质押
    // let latest_block = 19584321u64;  // Lido
    let block_range = BlockRange {
        from: latest_block,
        to: latest_block,
    };  
    let snapshot_date = "2024-06-01";
    let now_unix = chrono::Utc::now().timestamp() as u64;
    match scanner.run_once(block_range, snapshot_date, now_unix).await {
        Ok(inserted) => println!("Scan complete. New projects inserted: {}", inserted),
        Err(e) => eprintln!("Scan failed with error: {:?}", e),
    }
    Ok(())
}