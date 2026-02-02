use async_trait::async_trait;
use alloy::primitives::Address;
use super::{
    types::{ChainId, ContractCandidate, ProjectSnapshot, FilterDecision, 
        ProjectType, LockupType, AprSource, AdminKeyType},
    errors::ScanError,
    evm::EvmClient,
    config::ScannerConfig,
};  

// ========================== Codes ==========================

#[async_trait]
pub trait CandidateInspector: Send + Sync {
    async fn inspect(
        &self,
        cand: &ContractCandidate,
        decision: &FilterDecision,
        snapshot_date: &str,
        now_unix: u64,
    ) -> Result<ProjectSnapshot, ScanError>;
}

pub struct ShallowInspector<C: EvmClient> {
    pub cfg: ScannerConfig,
    pub client: C,
}

#[async_trait]
impl<C: EvmClient> CandidateInspector for ShallowInspector<C> {
    async fn inspect(
        &self,
        cand: &ContractCandidate,
        decision: &FilterDecision,
        snapshot_date: &str,
        now_unix: u64,
    ) -> Result<ProjectSnapshot, ScanError> {
        // v1: best-effort, many fields None/Unknown.
        let is_upgradeable = self.detect_proxy(cand).await.ok();

        let snap = ProjectSnapshot {
            project_id: make_project_id(cand.chain_id, &cand.contract),
            chain_id: cand.chain_id,

            staking_contract: cand.contract.clone(),
            staked_token: None,
            receipt_token: None,

            project_type: ProjectType::Unknown,
            issuer_name: None,

            lockup_type: LockupType::Unknown,
            lockup_duration_days: None,
            withdraw_delay_days: None,
            early_exit_penalty: None,

            reward_token: None,
            reward_rate_raw: None,
            apr_estimated: None,
            apr_source: AprSource::Unknown,

            tvl_token_amount: None,
            tvl_usd_estimated: None,
            top_1_holder_ratio: None,
            top_5_holder_ratio: None,

            is_upgradeable,
            admin_key_type: AdminKeyType::Unknown,
            can_pause: None,
            can_change_reward: None,

            has_slashing: None,
            principal_guaranteed: None,

            discovered_at_block: cand.deployed_block,
            discovered_at_time_unix: now_unix,
            snapshot_date: snapshot_date.to_string(),

            confidence_score: decision.confidence,
        };

        Ok(snap)
    }
}

impl<C: EvmClient> ShallowInspector<C> {
    async fn detect_proxy(
        &self,
        cand: &ContractCandidate,
    ) -> Result<bool, ScanError> {
        // PSEUDOCODE:
        // - check EIP-1967 implementation slot via eth_call (provider must support)
        // - or heuristically detect minimal proxy bytecode patterns
        // For skeleton, do bytecode heuristic.
        let code = self.client.get_code(cand.contract.clone(), Some(cand.deployed_block)).await?;
        if code.len() < 32 {
            return Ok(false);
        }
        // naive heuristic placeholder
        Ok(false)
    }
}

fn make_project_id(chain_id: ChainId, addr: &Address) -> String {
    let mut s = format!("{}:", chain_id);
    for b in addr.0 {
        s.push_str(&format!("{:02x}", b));
    }
    s
}