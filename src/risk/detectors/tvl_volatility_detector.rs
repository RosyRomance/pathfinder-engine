use crate::risk::engine::RiskDetector;
use async_trait::async_trait;
use crate::finder::types::ContractCandidate;
use super::super::{
    engine::RiskContext,
    scorer::RiskFlag,
};

// ========================== Codes ==========================

pub struct TvlVolatilityDetector {
    // e.g. 0.40 means 40% change triggers
    pub threshold_abs_change: f64,
}

#[async_trait]
impl RiskDetector for TvlVolatilityDetector {
    fn name(&self) -> &'static str { "tvl_volatility_24h" }
    fn order(&self) -> u32 { 200 }

    async fn detect(&self, ctx: &dyn RiskContext, cand: &ContractCandidate) -> Result<Vec<RiskFlag>, String> {
        let hist = ctx.tvl_history_24h(cand.contract)?;
        if hist.len() < 2 {
            return Ok(vec![]);
        }

        // assume hist[0] older, hist[last] now
        let old = hist.first().unwrap().tvl_usd;
        let now = hist.last().unwrap().tvl_usd;

        if old <= 0.0 {
            return Ok(vec![]);
        }

        let change = (now - old) / old; // -0.4 means -40%
        if change.abs() >= self.threshold_abs_change {
            Ok(vec![RiskFlag::TVLVolatilityHigh { change_24h: change }])
        } else {
            Ok(vec![])
        }
    }
}
