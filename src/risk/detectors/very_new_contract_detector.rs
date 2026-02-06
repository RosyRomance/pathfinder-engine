pub struct VeryNewContractDetector {
    pub max_age_blocks: u64,
}

impl RiskDetector for VeryNewContractDetector {
    fn name(&self) -> &'static str { "very_new_contract" }
    fn order(&self) -> u32 { 30 }

    fn detect(&self, ctx: &dyn RiskContext, target: Address) -> Result<Vec<RiskFlag>, String> {
        let age = ctx.contract_age_blocks(target)?;
        if age <= self.max_age_blocks {
            Ok(vec![RiskFlag::VeryNewContract { age_blocks: age }])
        } else {
            Ok(vec![])
        }
    }
}
