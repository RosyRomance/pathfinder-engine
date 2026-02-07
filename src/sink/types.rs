use serde::{Serialize, Deserialize};
use clickhouse::{Row, Client};
use std::str::FromStr;
use alloy::{
    primitives::{Address, },
    json_abi::JsonAbi,
};
use crate::{
    finder::{
        types::{ChainId, BlockNumber, ContractCandidate, },
        errors::ScanError,
    },
    risk::{
        apr_scanner::{AprType, RewardModel},
        scorer::{AprRiskFlag, RiskFlag}, 
    }
};

// ========================== Codes ==========================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    /// 快照 schema 版本（不是代码版本）
    pub schema_version: u32,

    /// 所属链
    pub chain_id: ChainId,

    /// 被分析的核心合约（staking / vault / gauge）
    pub staking_contract: Address,

    /// 本次快照基于的区块高度
    pub produced_at_block: BlockNumber,

    /// 扫描时间（wall clock，用于 UI / 排序）
    pub produced_at_ts: u64,

    /// 识别出的协议 / 奖励模型
    pub reward_model: Option<RewardModel>,

    /// APR 类型（语义）
    pub apr_type: Option<AprType>,

    /// 原子风险（事实）
    pub risk_flags: Vec<RiskFlag>,

    /// 派生风险（APR 语义）
    pub apr_risk_flags: Vec<AprRiskFlag>,

    /// 综合风险评分（0.0 ~ 1.0）
    pub risk_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Row)]
pub struct PendingContractRow {
    pub chain_id: u64,
    pub contract: String,
    pub deployed_block: u64,
    pub receives_token: Option<u8>,
    pub has_balance: Option<u8>,
    pub verify_attempts: u32,
    pub first_receive_block: Option<u64>,
    pub abi: String, // JSON string; empty = None
}

impl PendingContractRow {
    pub fn into_candidate(self) -> Result<ContractCandidate, ScanError> {
        let contract = parse_address(&self.contract)?;

        let abi = if self.abi.trim().is_empty() {
            None
        } else {
            let parsed: JsonAbi = serde_json::from_str(&self.abi)
                .map_err(|e| ScanError::Serde(format!("parse abi json failed: {e}")))?;
            Some(parsed)
        };

        Ok(ContractCandidate {
            chain_id: self.chain_id as ChainId,
            contract,
            deployed_block: self.deployed_block as BlockNumber,
            receives_token: opt_u8_to_opt_bool(self.receives_token),
            has_balance: opt_u8_to_opt_bool(self.has_balance),
            verify_attempts: self.verify_attempts,
            first_receive_block: self.first_receive_block.map(|v| v as BlockNumber),
            abi,
        })
    }

    pub fn from_candidate(c: &ContractCandidate) -> Result<Self, ScanError> {
        let contract = format!("{:#x}", c.contract);

        let abi = match &c.abi {
            None => String::new(),
            Some(a) => serde_json::to_string(a)
                .map_err(|e| ScanError::Serde(format!("serialize abi json failed: {e}")))?,
        };

        Ok(Self {
            chain_id: c.chain_id as u64,
            contract,
            deployed_block: c.deployed_block as u64,
            receives_token: opt_bool_to_opt_u8(c.receives_token),
            has_balance: opt_bool_to_opt_u8(c.has_balance),
            verify_attempts: c.verify_attempts,
            first_receive_block: c.first_receive_block.map(|v| v as u64),
            abi,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Row)]
pub struct VerifiedProjectRow {
    pub chain_id: u64,
    pub staking_contract: String,
    pub snapshot_json: String,
}

impl VerifiedProjectRow {
    pub fn into_snapshot(self) -> Result<ProjectSnapshot, ScanError> {
        let snap: ProjectSnapshot = serde_json::from_str(&self.snapshot_json)
            .map_err(|e| ScanError::Serde(format!("parse snapshot json failed: {e}")))?;

        // Optional sanity check:
        // if snap.chain_id as u64 != self.chain_id { ... }

        Ok(snap)
    }

    pub fn from_snapshot(s: &ProjectSnapshot) -> Result<Self, ScanError> {
        let staking_contract = format!("{:#x}", s.staking_contract);
        let snapshot_json = serde_json::to_string(s)
            .map_err(|e| ScanError::Serde(format!("serialize snapshot json failed: {e}")))?;

        Ok(Self {
            chain_id: s.chain_id as u64,
            staking_contract,
            snapshot_json,
        })
    }
}

pub struct ClickhouseClient {
    client: Client,
}

// ========================== funcs ==========================
fn parse_address(s: &str) -> Result<Address, ScanError> {
    Address::from_str(s).map_err(|e| ScanError::Store(format!("invalid address: {s}: {e}")))
}

fn opt_u8_to_opt_bool(v: Option<u8>) -> Option<bool> {
    v.map(|x| x != 0)
}

fn opt_bool_to_opt_u8(v: Option<bool>) -> Option<u8> {
    v.map(|b| if b { 1 } else { 0 })
}