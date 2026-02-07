use alloy::primitives::{Address, B256};
use crate::risk::engine::RiskDetector;
use async_trait::async_trait;
use crate::finder::types::ContractCandidate;
use super::super::{
    engine::RiskContext, 
    scorer::RiskFlag,
};

// ========================== Codes ==========================

pub struct ProxyAdminEoaDetector {
    // 这里用 EIP-1967 的 admin slot。slot 常量是固定的：
    // bytes32(uint256(keccak256("eip1967.proxy.admin")) - 1)
    pub eip1967_admin_slot: B256,
}

#[async_trait]
impl RiskDetector for ProxyAdminEoaDetector {
    fn name(&self) -> &'static str { "proxy_admin_eoa" }
    fn order(&self) -> u32 { 10 }

    async fn detect(&self, ctx: &dyn RiskContext, cand: &ContractCandidate) -> Result<Vec<RiskFlag>, String> {
        let raw = ctx.get_storage_at(cand.contract, self.eip1967_admin_slot).await?;
        // 转成 bytes32（大端）
        let word = B256::from(raw);

        // admin address is rightmost 20 bytes of the slot
        let admin = Address::from_slice(&word.as_slice()[12..32]);

        // 0x00..00 means not a proxy / slot empty
        if admin == Address::ZERO {
            return Ok(vec![]);
        }

        if ctx.is_eoa(admin).await? {
            Ok(vec![RiskFlag::ProxyAdminIsEOA])
        } else {
            Ok(vec![])
        }
    }
}
