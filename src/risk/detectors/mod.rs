pub mod holder_concentration_detector;
pub mod reward_inflation_detector;
pub mod very_new_contract_detector;
pub mod proxy_admin_eoa_detector;
pub mod owner_privilege_detector;
pub mod tvl_volatility_detector;
pub mod apr_risk_detector;


// pub fn build_default_risk_engine(
//     share_token: alloy_primitives::Address,
//     reward_token: Option<alloy_primitives::Address>,
//     eip1967_admin_slot: alloy_primitives::B256,
// ) -> RiskEngine {
//     let mut detectors: Vec<Box<dyn RiskDetector>> = Vec::new();

//     detectors.push(Box::new(ProxyAdminEoaDetector {
//         eip1967_admin_slot,
//     }));

//     detectors.push(Box::new(OwnerPrivilegeDetector));

//     detectors.push(Box::new(VeryNewContractDetector {
//         max_age_blocks: 20_000,
//     }));

//     detectors.push(Box::new(TvlVolatilityDetector {
//         threshold_abs_change: 0.40,
//     }));

//     detectors.push(Box::new(HolderConcentrationDetector {
//         top1_threshold: 0.50,
//         top3_threshold: 0.75,
//         share_token,
//     }));

//     if let Some(rt) = reward_token {
//         detectors.push(Box::new(RewardInflationDetector {
//             reward_token: rt,
//         }));
//     }

//     RiskEngine::new(
//         detectors,
//         Box::new(DefaultRiskScorer),
//     )
// }


// pub async fn inspect_risk(
//     &self,
//     cand: &ContractCandidate,
//     ctx: &dyn RiskContext,
// ) -> Result<RiskReport, ScanError> {
//     let engine = self.risk_engine(); // 你可以缓存在 self 里
//     engine.run(ctx, cand.contract).map_err(ScanError::Config)
// }

// helper
pub fn hex_to_bytes4(s: &str) -> [u8; 4] {
    let bytes = hex::decode(s).expect("invalid hex selector");
    [bytes[0], bytes[1], bytes[2], bytes[3]]
}