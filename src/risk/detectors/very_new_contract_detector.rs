use crate::risk::engine::RiskDetector;
use crate::finder::types::ContractCandidate;
use async_trait::async_trait;
use super::super::{
    engine::RiskContext,
    scorer::RiskFlag,
};

// ========================== Codes ==========================

pub struct VeryNewContractDetector {
    pub max_age_blocks: u64,
}

#[async_trait]
impl RiskDetector for VeryNewContractDetector {
    fn name(&self) -> &'static str { "very_new_contract" }
    fn order(&self) -> u32 { 30 }

    async fn detect(&self, ctx: &dyn RiskContext, cand: &ContractCandidate) -> Result<Vec<RiskFlag>, String> {
        let age = ctx.contract_age_blocks(cand.contract)?;
        if age <= self.max_age_blocks {
            Ok(vec![RiskFlag::VeryNewContract { age_blocks: age }])
        } else {
            Ok(vec![])
        }
    }
}
