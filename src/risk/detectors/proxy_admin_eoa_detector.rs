use alloy::primitives::{Address, B256};
use crate::risk::engine::RiskDetector;
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

impl RiskDetector for ProxyAdminEoaDetector {
    fn name(&self) -> &'static str { "proxy_admin_eoa" }
    fn order(&self) -> u32 { 10 }

    fn detect(&self, ctx: &dyn RiskContext, target: Address) -> Result<Vec<RiskFlag>, String> {
        let raw = ctx.get_storage_at(target, self.eip1967_admin_slot)?;

        // admin address is rightmost 20 bytes of the slot
        let admin = Address::from_slice(&raw.as_slice()[12..32]);

        // 0x00..00 means not a proxy / slot empty
        if admin == Address::ZERO {
            return Ok(vec![]);
        }

        if ctx.is_eoa(admin)? {
            Ok(vec![RiskFlag::ProxyAdminIsEOA])
        } else {
            Ok(vec![])
        }
    }
}
