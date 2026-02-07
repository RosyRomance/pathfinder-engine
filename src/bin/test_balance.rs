use anyhow::Result;
use clickhouse::Client;
use alloy::primitives::{address, Address};
use pathfinder::{
    finder::{
        self, alloy_evm::AlloyEvmClient, discovery::{SimpleDiscovery, ContractDiscovery}, filter::{BehaviorFilter, CandidateFilter}, 
        types::{BlockRange, ContractCandidate, TxHash},
        config::ScannerConfig,
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

    let block = 15951000;
    let cfg = ScannerConfig {
        chain_id: 1,
        min_confidence: 0.75,
        lookback_blocks: 1000,
        max_candidates_per_run: 100,
        retain_balance_min_blocks: 10,  
        stake_like_selectors: vec![],
        min_selector_hits: 1,
        max_verify_attempts: 1000,
    };

    let discover = SimpleDiscovery {
        chain_id: 1,
        client: evm_client.clone(),
    };

    let filter = BehaviorFilter {
        cfg,
        client: evm_client.clone(),
    };

    let decide = discover.discover(block).await?;
    let tx_hashes = decide.tx_hashes;
    println!("TxHashes: {:?}", tx_hashes);
    let new_candidates = decide.new_candidates;

    let contract = ContractCandidate {
        chain_id: 1,
        contract: address!("0xB753548F6E010e7e680BA186F9Ca1BdAB2E90cf2"),
        deployed_block: block,
        receives_token: None,
        has_balance: None,
        verify_attempts: 1,
        first_receive_block: None,
        abi: None,
    };

    filter.decide(&contract, &tx_hashes);
    // filter.decide(&contract, &[TxHash("0x1315e38a43e5414e1d37d901857a7e8dc3ae433fa36777bff123b5c7cfff0b97")]);



    Ok(())
}