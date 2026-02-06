pub struct AprRiskDetector;

impl RiskDetector for AprRiskDetector {
    fn name(&self) -> &'static str { "apr_risk" }
    fn order(&self) -> u32 { 20 }

    fn detect(&self, ctx: &dyn RiskContext, target: Address) -> Vec<RiskFlag> {
        // 关键点：尝试把 ctx 当成 AprContext
        let apr_ctx = match (ctx as &dyn std::any::Any).downcast_ref::<dyn AprContext>() {
            Some(v) => v,
            None => return vec![], // 不支持 APR，直接跳过
        };

        let apr = match apr_ctx.apr_scan(target) {
            Ok(v) => v,
            Err(_) => return vec![],
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

        flags
    }
}

