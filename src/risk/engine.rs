use alloy::primitives::{Address, B256, Bytes, U256};
use crate::finder::types::ContractCandidate;
use async_trait::async_trait;
use super::{
    scorer::{RiskFlag, RiskReport, RiskScorer},
    apr_scanner::AprContext,
};

// ========================== RiskContext ==========================

#[derive(Debug, Clone)]
pub struct SnapshotPoint {
    pub block_number: u64,
    pub tvl_usd: f64,
}

#[async_trait]
pub trait RiskContext: Send + Sync + std::any::Any {
    // --- chain meta ---
    fn chain_id(&self) -> u64;
    fn now_block(&self) -> u64;

    // --- contract / code ---
    async fn get_code(&self, addr: Address) -> Result<Bytes, String>;

    // EIP-1967 admin slot probing (if proxy)
    async fn get_storage_at(&self, addr: Address, slot: B256) -> Result<U256, String>;

    // Best-effort: 判断某地址是否 EOA (code len == 0)
    async fn is_eoa(&self, addr: Address) -> Result<bool, String>;

    // --- token / holders / tvl ---
    // 返回 share token (staking receipt / lp / share) 的 holder 集中度
    fn holder_concentration(&self, share_token: Address) -> Result<(f64, f64), String>;

    // 返回 tvl 的时间序列 (至少两个点: now vs 24h ago)
    fn tvl_history_24h(&self, target: Address) -> Result<Vec<SnapshotPoint>, String>;

    // 合约部署距今多少 blocks (需要你在 discovery 时保存部署块高, 或从 first seen 推断)
    fn contract_age_blocks(&self, target: Address) -> Result<u64, String>;

    /// 如果该 context 支持 APR 分析，返回 AprContext
    fn apr_ctx(&self) -> Option<&dyn AprContext> {
        None
    }
}

// ========================== RiskDetect ==========================

#[async_trait]
pub trait RiskDetector: Send + Sync {
    fn name(&self) -> &'static str;

    // 执行检测, 返回触发的 flags
    async fn detect(&self, ctx: &dyn RiskContext, cand: &ContractCandidate) -> Result<Vec<RiskFlag>, String>;

    // 用于控制执行顺序: 数值越小越早执行
    fn order(&self) -> u32 { 100 }
}

// ========================== RiskEngine ==========================

pub struct RiskEngine {
    detectors: Vec<Box<dyn RiskDetector>>,
    scorer: Box<dyn RiskScorer>,
}

impl RiskEngine {
    pub fn new(
        detectors: Vec<Box<dyn RiskDetector>>,
        scorer: Box<dyn RiskScorer>,
    ) -> Self {
        let mut d = detectors;
        d.sort_by_key(|x| x.order());
        Self { detectors: d, scorer }
    }

    pub async fn run(
        &self,
        ctx: &dyn RiskContext,
        cand: &ContractCandidate,
    ) -> Result<RiskReport, String> {
        let mut flags: Vec<RiskFlag> = Vec::new();

        for det in &self.detectors {
            let mut out = det.detect(ctx, cand).await?;
            flags.append(&mut out);
        }

        // 去重: 简单做法, 你也可以实现 Hash for RiskFlag 再用 HashSet
        flags = dedup_flags(flags);

        let score = self.scorer.score(&flags);

        Ok(RiskReport {
            chain_id: ctx.chain_id(),
            target: cand.contract,
            flags,
            score,
        })
    }
}

fn dedup_flags(mut v: Vec<RiskFlag>) -> Vec<RiskFlag> {
    // MVP 版: O(n^2) 去重, flags 数量很小问题不大
    let mut out: Vec<RiskFlag> = Vec::new();
    'outer: for f in v.drain(..) {
        for e in &out {
            if same_flag_kind(e, &f) {
                continue 'outer;
            }
        }
        out.push(f);
    }
    out
}

fn same_flag_kind(a: &RiskFlag, b: &RiskFlag) -> bool {
    match (a, b) {
        (RiskFlag::OwnerCanWithdraw, RiskFlag::OwnerCanWithdraw) => true,
        (RiskFlag::OwnerCanPause, RiskFlag::OwnerCanPause) => true,
        (RiskFlag::RewardParamsMutable, RiskFlag::RewardParamsMutable) => true,
        (RiskFlag::RewardTokenInflation, RiskFlag::RewardTokenInflation) => true,
        (RiskFlag::ProxyAdminIsEOA, RiskFlag::ProxyAdminIsEOA) => true,
        (RiskFlag::TVLConcentrationHigh { .. }, RiskFlag::TVLConcentrationHigh { .. }) => true,
        (RiskFlag::TVLVolatilityHigh { .. }, RiskFlag::TVLVolatilityHigh { .. }) => true,
        (RiskFlag::VeryNewContract { .. }, RiskFlag::VeryNewContract { .. }) => true,
        _ => false,
    }
}
