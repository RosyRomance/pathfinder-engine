use alloy::{
    primitives::{Address, B256, Bytes},
    providers::Provider,
};
use std::sync::Arc;

// ========================== Codes ==========================

pub struct AlloyRiskContext<P> {
    pub chain_id: u64,
    pub provider: Arc<P>,

    // 可选：项目存储，用于 TVL / holder / 历史
    pub store: Arc<dyn RiskStore>,

    // 当前扫描用的区块高度
    pub now_block: u64,
}

pub trait RiskStore: Send + Sync {
    // holder concentration of share token
    // return (top1_ratio, top3_ratio)
    fn holder_concentration(
        &self,
        token: Address,
    ) -> Result<(f64, f64), String>;

    // tvl history points (older -> newer)
    fn tvl_history_24h(
        &self,
        target: Address,
    ) -> Result<Vec<SnapshotPoint>, String>;

    // contract age in blocks
    fn contract_age_blocks(
        &self,
        target: Address,
        now_block: u64,
    ) -> Result<u64, String>;
}

// ========================== Codes ==========================

impl<P> RiskContext for AlloyRiskContext<P>
where
    P: Provider + Send + Sync,
{
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn now_block(&self) -> u64 {
        self.now_block
    }

    // -----------------------------
    // code / storage
    // -----------------------------

    fn get_code(&self, addr: Address) -> Result<Bytes, String> {
        self.provider
            .get_code_at(addr)
            .map_err(|e| e.to_string())
    }

    fn get_storage_at(&self, addr: Address, slot: B256) -> Result<B256, String> {
        self.provider
            .get_storage_at(addr, slot)
            .map_err(|e| e.to_string())
    }

    fn is_eoa(&self, addr: Address) -> Result<bool, String> {
        let code = self.get_code(addr)?;
        Ok(code.is_empty())
    }

    // -----------------------------
    // economic data (delegated to store)
    // -----------------------------

    fn holder_concentration(
        &self,
        share_token: Address,
    ) -> Result<(f64, f64), String> {
        self.store.holder_concentration(share_token)
    }

    fn tvl_history_24h(
        &self,
        target: Address,
    ) -> Result<Vec<SnapshotPoint>, String> {
        self.store.tvl_history_24h(target)
    }

    fn contract_age_blocks(
        &self,
        target: Address,
    ) -> Result<u64, String> {
        self.store.contract_age_blocks(target, self.now_block)
    }
}

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
