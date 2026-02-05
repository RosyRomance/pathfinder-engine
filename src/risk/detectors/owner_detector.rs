
// 4-byte selectors (hex, no 0x prefix)
const OWNER_WITHDRAW_SELECTORS: &[&str] = &[
    "2e1a7d4d", // withdraw(uint256)
    "853828b6", // emergencyWithdraw()
    "3ccfd60b", // withdrawAll()
    "9e281a98", // sweep(address)
    "a9059cbb", // transfer(address,uint256) (owner gated)
];

const PAUSE_SELECTORS: &[&str] = &[
    "8456cb59", // pause()
    "3f4ba83a", // unpause()
];

const REWARD_MUTABLE_SELECTORS: &[&str] = &[
    "8bdb3913", // setRewardRate(uint256)
    "f2fde38b", // transferOwnership(address)
    "d4ee1d90", // updateEmissionRate(uint256)
];


pub struct OwnerPrivilegeDetector;

impl RiskDetector for OwnerPrivilegeDetector {
    fn name(&self) -> &'static str { "owner_privilege" }
    fn order(&self) -> u32 { 20 }

    fn detect(
        &self,
        ctx: &dyn RiskContext,
        target: alloy_primitives::Address,
    ) -> Result<Vec<RiskFlag>, String> {
        let code = ctx.get_code(target)?;
        if code.is_empty() {
            return Ok(vec![]);
        }

        let mut flags: Vec<RiskFlag> = Vec::new();

        let has = |sel: &str| -> bool {
            let needle = hex_to_bytes4(sel);
            code.windows(4).any(|w| w == needle)
        };

        // R1: owner withdraw
        if OWNER_WITHDRAW_SELECTORS.iter().any(|s| has(s)) {
            flags.push(RiskFlag::OwnerCanWithdraw);
        }

        // R2: pause
        if PAUSE_SELECTORS.iter().any(|s| has(s)) {
            flags.push(RiskFlag::OwnerCanPause);
        }

        // R7: reward mutable
        if REWARD_MUTABLE_SELECTORS.iter().any(|s| has(s)) {
            flags.push(RiskFlag::RewardParamsMutable);
        }

        Ok(flags)
    }
}

// helper
fn hex_to_bytes4(s: &str) -> [u8; 4] {
    let bytes = hex::decode(s).expect("invalid hex selector");
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}
