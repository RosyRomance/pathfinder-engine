

// ========================== Codes ==========================

#[derive(Debug, Clone)]
pub struct RiskReport {
    pub chain_id: u64,
    pub target: alloy_primitives::Address,
    pub flags: Vec<RiskFlag>,
    pub score: f64,
}

#[derive(Debug, Clone)]
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
    ProxyAdminIsEOA,
    UnlockDelayTooShort { seconds: u64 },
    NoMinimumLock,
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
            };
        }
        if s > 1.0 { 1.0 } else { s }
    }
}