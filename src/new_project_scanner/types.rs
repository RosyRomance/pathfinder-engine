use serde::{Serialize, Deserialize};
pub use alloy::primitives::Address;


pub type ChainId = u64;
pub type BlockNumber = u64;

// #[derive(Clone, Debug)]
// pub struct Address(pub [u8; 20]);

#[derive(Clone, Debug)]
pub struct TxHash(pub [u8; 32]);

#[derive(Clone, Debug)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<[u8; 32]>,
    pub data: Vec<u8>,
    pub block_number: BlockNumber,
    pub tx_hash: TxHash,
}

#[derive(Clone, Debug)]
pub struct BlockRange {
    pub from: BlockNumber,
    pub to: BlockNumber, // inclusive or exclusive, choose one and be consistent
}

#[derive(Clone, Debug)]
pub struct ContractCandidate {
    pub chain_id: ChainId,
    pub contract: Address,
    pub deployed_block: BlockNumber,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProjectType {
    Unknown,
    Native,
    Lst,
    Lp,
    Restaking,
    Other,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LockupType {
    None,
    Soft,
    Hard,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AprSource {
    Onchain,
    Api,
    Inferred,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AdminKeyType {
    None,
    Multisig,
    SingleSig,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub project_id: String,
    pub chain_id: ChainId,

    pub staking_contract: Address,
    pub staked_token: Option<Address>,
    pub receipt_token: Option<Address>,

    pub project_type: ProjectType,
    pub issuer_name: Option<String>,

    pub lockup_type: LockupType,
    pub lockup_duration_days: Option<u32>,
    pub withdraw_delay_days: Option<u32>,
    pub early_exit_penalty: Option<bool>,

    pub reward_token: Option<Address>,
    pub reward_rate_raw: Option<String>,
    pub apr_estimated: Option<f64>,
    pub apr_source: AprSource,

    pub tvl_token_amount: Option<String>,
    pub tvl_usd_estimated: Option<f64>,
    pub top_1_holder_ratio: Option<f64>,
    pub top_5_holder_ratio: Option<f64>,

    pub is_upgradeable: Option<bool>,
    pub admin_key_type: AdminKeyType,
    pub can_pause: Option<bool>,
    pub can_change_reward: Option<bool>,

    pub has_slashing: Option<bool>,
    pub principal_guaranteed: Option<bool>,

    pub discovered_at_block: BlockNumber,
    pub discovered_at_time_unix: u64,
    pub snapshot_date: String,

    pub confidence_score: f64,
}

#[derive(Clone, Debug)]
pub struct FilterSignals {
    pub receives_token: bool,
    pub retains_balance: bool,
    pub has_stake_like_methods: bool,
}

#[derive(Clone, Debug)]
pub struct FilterDecision {
    pub pass: bool,
    pub confidence: f64,
    pub signals: FilterSignals,
}