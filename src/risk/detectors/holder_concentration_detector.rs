pub struct HolderConcentrationDetector {
    pub top1_threshold: f64,
    pub top3_threshold: f64,
    // share token: staking receipt / lp / share token
    pub share_token: alloy_primitives::Address,
}

impl RiskDetector for HolderConcentrationDetector {
    fn name(&self) -> &'static str { "holder_concentration" }
    fn order(&self) -> u32 { 150 }

    fn detect(
        &self,
        ctx: &dyn RiskContext,
        _target: alloy_primitives::Address,
    ) -> Result<Vec<RiskFlag>, String> {
        let (top1, top3) = ctx.holder_concentration(self.share_token)?;

        if top1 >= self.top1_threshold || top3 >= self.top3_threshold {
            Ok(vec![RiskFlag::TVLConcentrationHigh {
                top1,
                top3,
            }])
        } else {
            Ok(vec![])
        }
    }
}

// 设计说明（重要）

// share_token 必须明确

// staking: receipt token

// LP staking: LP token

// 不要直接用 underlying token（ETH / USDC），那是噪音

// 这个 detector 非常适合：

// 新项目

// LP staking

// 高 APY farming