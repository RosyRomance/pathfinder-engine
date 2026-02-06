use alloy::primitives::Address;
use crate::risk::engine::RiskDetector;
use super::super::{
    engine::RiskContext, 
    scorer::RiskFlag,
    apr::AprType,
    context::AprContext,
};

// ========================== Codes ==========================

pub struct AprRiskDetector;

impl RiskDetector for AprRiskDetector {
    fn name(&self) -> &'static str { "apr_risk" }
    fn order(&self) -> u32 { 20 }

    fn detect(&self, ctx: &dyn RiskContext, target: Address) -> Result<Vec<RiskFlag>, String> {
        // 关键点：尝试把 ctx 当成 AprContext
        let apr_ctx = match (ctx as &dyn std::any::Any).downcast_ref::<dyn AprContext>() {
            Some(v) => v,
            None => return Ok(vec![]), // 不支持 APR，直接跳过
        };

        let apr = match apr_ctx.apr_scan(target) {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut flags = vec![];

        if apr.modifiable {
            flags.push(RiskFlag::AprOwnerModifiable);
        }

        if let Some(days) = apr.reward_remaining_days {
            if days < 30.0 {
                flags.push(RiskFlag::AprShortLived);
            }
        }

        if matches!(apr.apr_type, AprType::Inflationary) {
            flags.push(RiskFlag::AprInflationary);
        }

        Ok(flags)
    }
}

