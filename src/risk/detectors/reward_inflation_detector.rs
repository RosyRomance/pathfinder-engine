use alloy::primitives::Address;
use crate::risk::engine::RiskDetector;
use super::super::{
    engine::RiskContext,
    scorer::RiskFlag,
};

// ========================== Codes ==========================

pub struct RewardInflationDetector {
    pub reward_token: Address,
}

impl RiskDetector for RewardInflationDetector {
    fn name(&self) -> &'static str { "reward_inflation" }
    fn order(&self) -> u32 { 120 }

    fn detect(
        &self,
        ctx: &dyn RiskContext,
        _target: Address,
    ) -> Result<Vec<RiskFlag>, String> {
        let code = ctx.get_code(self.reward_token)?;
        if code.is_empty() {
            return Ok(vec![]);
        }

        // mint(address,uint256)
        let mint_selector = super::hex_to_bytes4("40c10f19");

        if code.windows(4).any(|w| w == mint_selector) {
            Ok(vec![RiskFlag::RewardTokenInflation])
        } else {
            Ok(vec![])
        }
    }
}
