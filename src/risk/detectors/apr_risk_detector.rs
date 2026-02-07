use crate::risk::engine::RiskDetector;
use async_trait::async_trait;
use crate::finder::types::ContractCandidate;
use super::super::{
    engine::RiskContext, 
    scorer::{RiskFlag, AprRiskFlag},
    apr_scanner::{AprType, AprContext},
};

// ========================== Codes ==========================

pub struct AprRiskDetector;

#[async_trait]
impl RiskDetector for AprRiskDetector {
    fn name(&self) -> &'static str { "apr_risk" }
    fn order(&self) -> u32 { 20 }

    async fn detect(&self, ctx: &dyn RiskContext, cand: &ContractCandidate) -> Result<Vec<RiskFlag>, String> {
        // 关键点：尝试把 ctx 当成 AprContext
        let apr_ctx = match ctx.apr_ctx() {
            Some(v) => v,
            None => return Ok(vec![]), // 不支持 APR，直接跳过
        };

        let apr = match apr_ctx.apr_scan(cand).await {
            Ok(v) => v,
            Err(_) => return Ok(vec![]),
        };

        let mut flags = vec![];

        if apr.modifiable {
            flags.push(RiskFlag::Apr(AprRiskFlag::OwnerModifiable));
        }

        if let Some(days) = apr.reward_remaining_days {
            if days < 30.0 {
                flags.push(RiskFlag::Apr(AprRiskFlag::ShortLived));
            }
        }

        if matches!(apr.apr_type, AprType::Inflationary) {
            flags.push(RiskFlag::Apr(AprRiskFlag::Inflationary));
        }

        Ok(flags)
    }
}

