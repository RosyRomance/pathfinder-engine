use alloy::primitives::Address;
use serde::{Deserialize, Serialize};

// ========================== Codes ==========================

#[derive(Debug, Clone)]
pub struct RiskReport {
    pub chain_id: u64,
    pub target: Address,
    pub flags: Vec<RiskFlag>,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskFlag {
    // --- Static / 权限类 ---
    OwnerCanWithdraw,
    OwnerCanPause,
    RewardParamsMutable,
    RewardTokenInflation,
    ProxyAdminIsEOA,

    // --- Dynamic / 资金行为类 ---
    TVLConcentrationHigh { top1: f64, top3: f64 },
    TVLVolatilityHigh { change_24h: f64 },

    // --- Time / 历史类 ---
    VeryNewContract { age_blocks: u64 },

    // --- Other / 其他 ---
    UnlockDelayTooShort { seconds: u64 },
    NoMinimumLock,

    // --- Derived / APR 语义类 ---
    Apr(AprRiskFlag),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AprRiskFlag {
    OwnerModifiable,
    ShortLived,
    Inflationary,
    VeryHighApr,
}

pub trait RiskScorer: Send + Sync {
    fn score(&self, flags: &[RiskFlag]) -> f64;
}

pub struct DefaultRiskScorer;

impl RiskScorer for DefaultRiskScorer {
    fn score(&self, flags: &[RiskFlag]) -> f64 {
        let mut s = 0.0f64;
        for f in flags {
            s += match f {
                RiskFlag::OwnerCanWithdraw => 0.35,
                RiskFlag::TVLConcentrationHigh { .. } => 0.25,
                RiskFlag::TVLVolatilityHigh { .. } => 0.20,
                RiskFlag::ProxyAdminIsEOA => 0.15,
                RiskFlag::OwnerCanPause => 0.10,
                RiskFlag::RewardParamsMutable => 0.10,
                RiskFlag::RewardTokenInflation => 0.08,
                RiskFlag::VeryNewContract { .. } => 0.05,
                RiskFlag::UnlockDelayTooShort { .. } => 0.05,
                RiskFlag::NoMinimumLock => 0.05,

                // ---------- APR 派生风险 ----------
                RiskFlag::Apr(apr) => match apr {
                    AprRiskFlag::OwnerModifiable => 0.10,
                    AprRiskFlag::ShortLived => 0.08,
                    AprRiskFlag::Inflationary => 0.06,
                    AprRiskFlag::VeryHighApr => 0.05,
                },
            };
        }
        if s > 1.0 { 1.0 } else { s }
    }
}