use alloy::{
    primitives::{Address, B256, Bytes, U256},
    providers::Provider,
};
use std::sync::Arc;
use async_trait::async_trait;
use crate::finder::evm::EvmClient;
use super::{
    apr_scanner::{AprScanResult, AprContext},
    engine::{
        SnapshotPoint,
        RiskContext,
    },
};

// ========================== Codes ==========================

pub struct AlloyRiskContext<C: EvmClient> {
    pub chain_id: u64,
    pub client: C,

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

#[async_trait]
impl<C> RiskContext for AlloyRiskContext<C>
where
    C: 'static + EvmClient + Send + Sync,
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

    async fn get_code(&self, addr: Address) -> Result<Bytes, String> {
        self.client.provider()
            .get_code_at(addr)
            .await
            .map_err(|e| e.to_string())
    }

    async fn get_storage_at(&self, addr: Address, slot: B256) -> Result<U256, String> {
        self.client.provider()
            .get_storage_at(addr, slot.into())
            .await
            .map_err(|e| e.to_string())
    }

    async fn is_eoa(&self, addr: Address) -> Result<bool, String> {
        let code = self.get_code(addr).await?;
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

    fn apr_ctx(&self) -> Option<&dyn AprContext> {
        Some(self)
    }
}


// let ctx = AlloyRiskContext {
//     chain_id: self.cfg.chain_id,
//     provider: self.provider.clone(),
//     store: self.risk_store.clone(),
//     now_block,
// };

// let engine = build_default_risk_engine(
//     cand.share_token,
//     cand.reward_token,
//     EIP1967_ADMIN_SLOT,
// );

// let report = engine.run(&ctx, cand.contract)?;


// ========================== Codes ==========================
