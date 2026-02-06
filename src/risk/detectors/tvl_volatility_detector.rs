use alloy::primitives::Address;
use crate::risk::engine::RiskDetector;
use super::super::{
    engine::RiskContext,
    context::RiskStore,
    scorer::RiskFlag,
};

// ========================== Codes ==========================

pub struct TvlVolatilityDetector {
    // e.g. 0.40 means 40% change triggers
    pub threshold_abs_change: f64,
}

impl RiskDetector for TvlVolatilityDetector {
    fn name(&self) -> &'static str { "tvl_volatility_24h" }
    fn order(&self) -> u32 { 200 }

    fn detect(&self, ctx: &dyn RiskContext, target: Address) -> Result<Vec<RiskFlag>, String> {
        let hist = ctx.tvl_history_24h(target)?;
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
