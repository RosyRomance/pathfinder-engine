use anyhow::Result;
use std::sync::Arc;
use crate::{
    finder::{
        evm::EvmClient,
        inspect::ShallowInspector,
        discovery::SimpleDiscovery, 
        config::ScannerConfig,
        filter::BehaviorFilter,
        scanner::Scanner,
    },
    sink::project_store::ClickhouseStore,
};

// ===================== Demo main =====================

pub fn build<C>(store: Arc<ClickhouseStore>, client: C,
) -> Result<Scanner<
    SimpleDiscovery<C>,
    BehaviorFilter<C>,
    ShallowInspector<C>,
    ClickhouseStore,
>> 
where 
    C: EvmClient + Send + Sync + Clone + 'static,
{
    let config = ScannerConfig {
        chain_id: 1,
        min_confidence: 0.75,
        lookback_blocks: 1000,
        max_candidates_per_run: 100,
        retain_balance_min_blocks: 3,  
        stake_like_selectors: vec![
            // classic staking
            "4e71d92d".to_string(), "2e1a7d4d".to_string(), "3d18b912".to_string(), "e9fad8ee".to_string(),

            // ERC-4626
            "6e553f65".to_string(), "94bf804d".to_string(), "b460af94".to_string(), "ba087652".to_string(),

            // MasterChef / LP staking
            "e2bbb158".to_string(), "441a3e70".to_string(), "5312ea8e".to_string(),

            // LST / Restaking
            "a1903eab".to_string(), "ccee5c2f".to_string(),

            // vote escrow / lock
            "3f3f8f96".to_string(), "219f5d17".to_string(), "a5f3c23b".to_string(),

            // delegate / bond
            "5c19a95c".to_string(), "4c1f4d3a".to_string(),
        ],
        min_selector_hits: 1,
        max_verify_attempts: 1000,
    };

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

    let scanner = Scanner {
        cfg: config,
        discovery,
        filter,
        inspector,
        store,
    };

    Ok(scanner) 
}