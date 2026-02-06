
// ========================== Codes ==========================

use std::collections::HashMap;

pub struct MemoryRiskStore {
    // token -> (top1, top3)
    pub holder_map: HashMap<Address, (f64, f64)>,

    // contract -> tvl history
    pub tvl_map: HashMap<Address, Vec<SnapshotPoint>>,

    // contract -> deploy block
    pub deploy_block: HashMap<Address, u64>,
}

impl RiskStore for MemoryRiskStore {
    fn holder_concentration(
        &self,
        token: Address,
    ) -> Result<(f64, f64), String> {
        self.holder_map
            .get(&token)
            .copied()
            .ok_or_else(|| "no holder data".to_string())
    }

    fn tvl_history_24h(
        &self,
        target: Address,
    ) -> Result<Vec<SnapshotPoint>, String> {
        self.tvl_map
            .get(&target)
            .cloned()
            .ok_or_else(|| "no tvl history".to_string())
    }

    fn contract_age_blocks(
        &self,
        target: Address,
        now_block: u64,
    ) -> Result<u64, String> {
        let deploy = self
            .deploy_block
            .get(&target)
            .copied()
            .ok_or_else(|| "unknown deploy block".to_string())?;
        Ok(now_block.saturating_sub(deploy))
    }
}

