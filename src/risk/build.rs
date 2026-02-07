use alloy::{
    primitives::B256,
    hex::FromHex,
};
use once_cell::sync::Lazy;
use crate::{
    sink::types::ProjectSnapshot,
    finder::types::ProjectProfile,
    risk::detectors::{
        holder_concentration_detector::HolderConcentrationDetector, 
        owner_privilege_detector::OwnerPrivilegeDetector,
        proxy_admin_eoa_detector::ProxyAdminEoaDetector,
        reward_inflation_detector::RewardInflationDetector,
        tvl_volatility_detector::TvlVolatilityDetector,
        very_new_contract_detector::VeryNewContractDetector,
        apr_risk_detector::AprRiskDetector,
    },   
    risk::{
        engine::{RiskDetector, RiskEngine},
        scorer::DefaultRiskScorer, 
    },
};

/// EIP-1967 admin slot:
/// bytes32(uint256(keccak256("eip1967.proxy.admin")) - 1)
pub static EIP1967_ADMIN_SLOT: Lazy<B256> = Lazy::new(|| {
    B256::from_hex(
        "b53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103"
    ).expect("valid hex")
});

// ========================== Codes ==========================

pub fn build (snapshots: &Vec<ProjectSnapshot>, profiles: &Vec<ProjectProfile>) -> RiskEngine {
    // let snapshots: Vec<ProjectSnapshot> = Vec::new();
    // let profiles: Vec<ProjectProfile> = Vec::new();

    let mut detectors: Vec<Box<dyn RiskDetector>> = Vec::new();
    for snap in snapshots {
        detectors.push(Box::new( OwnerPrivilegeDetector));
        detectors.push(Box::new( ProxyAdminEoaDetector { eip1967_admin_slot: *EIP1967_ADMIN_SLOT }));
        detectors.push(Box::new( TvlVolatilityDetector { threshold_abs_change: 0.4 }));
        detectors.push(Box::new( VeryNewContractDetector { max_age_blocks: 7_200 }));  // 12s/block
        detectors.push(Box::new( AprRiskDetector ));

        if let Some(staked_token) = snap.staked_token {
            detectors.push(Box::new( HolderConcentrationDetector { top1_threshold: 0.3, top3_threshold: 0.6, share_token: staked_token }));
        }

        if let Some(reward_token) = snap.reward_token {
            detectors.push(Box::new( RewardInflationDetector { reward_token: reward_token }));
        }
    }

    for profile in profiles {
        detectors.push(Box::new( OwnerPrivilegeDetector));
        detectors.push(Box::new( ProxyAdminEoaDetector { eip1967_admin_slot: *EIP1967_ADMIN_SLOT }));
        detectors.push(Box::new( TvlVolatilityDetector { threshold_abs_change: 0.4 }));
        detectors.push(Box::new( VeryNewContractDetector { max_age_blocks: 7_200 }));  // 12s/block
        detectors.push(Box::new( AprRiskDetector ));

        if let Some(staked_token) = profile.staked_token {
            detectors.push(Box::new( HolderConcentrationDetector { top1_threshold: 0.3, top3_threshold: 0.6, share_token: staked_token }));
        }

        if let Some(reward_token) = profile.reward_token {
            detectors.push(Box::new( RewardInflationDetector { reward_token: reward_token }));
        }
    }

    let scorer = DefaultRiskScorer;
    let engine = RiskEngine::new(detectors, Box::new(scorer));

    engine
}