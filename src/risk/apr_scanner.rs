use alloy::{
    primitives::{Address, U256, Bytes, },
    json_abi::{JsonAbi, Function,},
    dyn_abi::{DynSolValue, SolType, JsonAbiExt},
    rpc::types::{TransactionRequest, TransactionInput, BlockNumberOrTag, BlockId},
    providers::{Provider, DynProvider},
    sol_types::sol_data::Uint, 
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::finder::{
    evm::EvmClient,
    errors::ScanError,
    types::ContractCandidate,
};
use super::{
    context::AlloyRiskContext,
    scorer::AprRiskFlag,
};


// ========================== Codes ==========================

#[async_trait]
pub trait AprContext {
    async fn apr_scan(&self, cand: &ContractCandidate) -> Result<AprScanResult, ScanError>;
}

#[async_trait]
impl<C> AprContext for AlloyRiskContext<C> 
where
    C: EvmClient + Send + Sync + 'static,
{
    async fn apr_scan(
        &self,
        cand: &ContractCandidate,
    ) -> Result<AprScanResult, ScanError> {
        // 1️⃣ 识别奖励模型
        let model = self.detect_reward_model(cand).await;

        // 2️⃣ 读取奖励参数
        let reward_info = self.read_reward_info(cand, &model).await?;

        // 3️⃣ 可持续性分析
        let sustain = self.analyze_sustainability(cand, &reward_info).await?;

        // 4️⃣ 风险标记
        let risk_flags = self.evaluate_risk(&reward_info, &sustain);

        Ok(AprScanResult {
            nominal_apr: reward_info.apr,
            apr_type: model.apr_type(),
            reward_tokens: reward_info.reward_tokens,
            reward_rate: reward_info.reward_rate,
            reward_remaining_days: sustain.remaining_days,
            modifiable: reward_info.modifiable,
            owner: reward_info.owner,
            risk_flags,
        })
    }
}

// ========================== Codes ==========================

#[derive(Debug, Clone)]
pub struct AprScanResult {
    pub nominal_apr: Option<f64>,     // 可以算就填，算不了就 None
    pub apr_type: AprType,             // 通胀 / 手续费 / 混合 / 固定承诺
    pub reward_tokens: Vec<Address>,

    pub reward_rate: Option<u128>,     // 每秒 / 每块
    pub reward_remaining_days: Option<f64>,

    pub modifiable: bool,              // 是否可被 owner 修改
    pub owner: Option<Address>,

    pub risk_flags: Vec<AprRiskFlag>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AprType {
    FixedEmission,     // 固定发放速率（StakingRewards）
    Inflationary,      // 通胀模型（MasterChef）
    Variable,          // 可变 / Gauge / Vote
    ExternalYield,     // 外部真实收益（手续费分成）
    Unknown,
}

// #[derive(Debug, Clone)]
// pub enum AprRiskFlag {
//     ShortLivedRewards,
//     OwnerCanModifyRewards,
//     VeryHighApr,
//     AprWithoutCap,
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RewardModel {
    MasterChef { pid: u64 },
    StakingRewards,
    // Gauge,
    Unknown,
}

impl RewardModel {
    pub fn apr_type(&self) -> AprType {
        match self {
            RewardModel::StakingRewards => AprType::FixedEmission,
            RewardModel::MasterChef { .. } => AprType::Inflationary,
            // RewardModel::Gauge => AprType::Variable,
            RewardModel::Unknown => AprType::Unknown,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RewardInfo {
    pub apr: Option<f64>,
    pub reward_rate: Option<u128>,          // per second or per block
    pub reward_tokens: Vec<Address>,
    pub reward_remaining_days: Option<f64>,
    pub modifiable: bool,
    pub owner: Option<Address>,
}

pub struct SustainResult {
    pub remaining_days: Option<f64>,
}


// ========================== Codes ==========================

impl<C: EvmClient> AlloyRiskContext<C> 
where
    C: EvmClient + Send + Sync + 'static,
{
    // pub async fn inspect(
    //     &self,
    //     cand: &ContractCandidate,
    // ) -> Result<AprScanResult, ScanError> {
    //     // 1️⃣ 识别奖励模型
    //     let model = self.detect_reward_model(cand).await?;

    //     // 2️⃣ 读取奖励参数
    //     let reward_info = self.read_reward_info(cand, &model).await?;

    //     // 3️⃣ 可持续性分析
    //     let sustain = self.analyze_sustainability(cand, &reward_info).await?;

    //     // 4️⃣ 风险标记
    //     let risk_flags = self.evaluate_risk(&reward_info, &sustain);

    //     Ok(AprScanResult {
    //         nominal_apr: reward_info.apr,
    //         apr_type: model.apr_type(),
    //         reward_tokens: reward_info.reward_tokens,
    //         reward_rate: reward_info.reward_rate,
    //         reward_remaining_days: sustain.remaining_days,
    //         modifiable: reward_info.modifiable,
    //         owner: reward_info.owner,
    //         risk_flags,
    //     })
    // }

    pub async fn detect_reward_model(
        &self,
        cand: &ContractCandidate,
    ) -> RewardModel {
        let abi = match &cand.abi {
            Some(a) => a,
            None => return RewardModel::Unknown,
        };

        // --- 1️⃣ StakingRewards ---
        if is_staking_rewards_abi(abi) {
            return RewardModel::StakingRewards;
        }

        // --- 2️⃣ MasterChef ---
        if is_masterchef_abi(abi) {
            // 先默认 pid = 0
            return RewardModel::MasterChef { pid: 0 };
        }

        RewardModel::Unknown
    }

    pub async fn read_reward_info(
        &self,
        cand: &ContractCandidate,
        model: &RewardModel,
    ) -> Result<RewardInfo, ScanError> {
        match model {
            RewardModel::StakingRewards => {
                self.read_staking_rewards(cand.contract).await
            }

            RewardModel::MasterChef { pid } => {
                self.read_masterchef_pool(cand.contract, *pid).await
            }

            RewardModel::Unknown => Ok(RewardInfo {
                apr: None,
                reward_rate: None,
                reward_tokens: vec![],
                reward_remaining_days: None,
                modifiable: false,
                owner: None,
            }),
        }
    }

    async fn read_staking_rewards(
        &self,
        addr: Address,
    ) -> Result<RewardInfo, ScanError> {
        // --- rewardRate() ---
        let reward_rate: Option<U256> = eth_call_view::<Uint<256>> (
            &self.client.provider(),
            addr,
            "function rewardRate() view returns (uint256)",
            &[],
        )
        .await;

        // --- periodFinish() ---
        let period_finish: Option<U256> = eth_call_view::<Uint<256>> (
            &self.client.provider(),
            addr,
            "function periodFinish() view returns (uint256)",
            &[],
        )
        .await;

        // --- rewardsToken() ---
        let reward_token: Option<Address> = eth_call_view::<alloy::sol_types::sol_data::Address> (
            &self.client.provider(),
            addr,
            "function rewardsToken() view returns (address)",
            &[],
        )
        .await;

        // --- owner() ---
        let owner: Option<Address> = eth_call_view::<alloy::sol_types::sol_data::Address> (
            &self.client.provider(),
            addr,
            "function owner() view returns (address)",
            &[],
        )
        .await;

        // --- 可持续性估算（基于 periodFinish） ---
        let remaining_days = if let (Some(rate), Some(finish)) =
            (reward_rate, period_finish)
        {
            let provider = self.client.provider();

            // 取最新区块
            let block = provider
                .get_block(BlockId::Number(BlockNumberOrTag::Latest))
                .await
                .ok()
                .flatten()
                .expect(" get latest block number ERROR");

            // block.timestamp 是 U256 / u64（取决于 network）
            let now: u64 = block.header.timestamp;
            let finish = finish.to::<u64>();

            if finish > now && !rate.is_zero() {
                Some(((finish - now) as f64) / 86400.0)
            } else {
                None
            }
        } else {
            None
        };

        Ok(RewardInfo {
            apr: None,
            reward_rate: reward_rate.map(|v| v.to::<u128>()),
            reward_tokens: reward_token.into_iter().collect(),
            reward_remaining_days: remaining_days,
            modifiable: owner.is_some(),
            owner,
        })
    }

    async fn read_masterchef_pool(
        &self,
        chef: Address,
        pid: u64,
    ) -> Result<RewardInfo, ScanError> {
        // --- rewardPerBlock() ---
        let reward_per_block: Option<U256> = eth_call_view::<Uint<256>> (
            &self.client.provider(),
            chef,
            "function rewardPerBlock() view returns (uint256)",
            &[],
        )
        .await;

        // --- totalAllocPoint() ---
        let total_alloc: Option<U256> = eth_call_view::<Uint<256>> (
            &self.client.provider(),
            chef,
            "function totalAllocPoint() view returns (uint256)",
            &[],
        )
        .await;

        // --- poolInfo(pid) ---
        let pool: Option<(Address, U256)> = eth_call_view::<(alloy::sol_types::sol_data::Address, Uint<256>)> (
            &self.client.provider(),
            chef,
            "function poolInfo(uint256) view returns (address lpToken, uint256 allocPoint)",
            &[DynSolValue::Uint(U256::from(pid), 256)],
        )
        .await;

        // --- owner() ---
        let owner: Option<Address> = eth_call_view::<alloy::sol_types::sol_data::Address> (
            &self.client.provider(),
            chef,
            "function owner() view returns (address)",
            &[],
        )
        .await;

        // --- 计算该 pool 的 reward rate ---
        let pool_reward = match (reward_per_block, total_alloc, pool.as_ref()) {
            (Some(rpb), Some(total), Some((_, alloc))) if !total.is_zero() => {
                Some((rpb * *alloc / total).to::<u128>())
            }
            _ => None,
        };

        Ok(RewardInfo {
            apr: None,
            reward_rate: pool_reward,    // per block
            reward_tokens: vec![],       // MasterChef 通常是协议 token
            reward_remaining_days: None, // 需单独算奖励池余额
            modifiable: owner.is_some(),
            owner,
        })
    }

    pub async fn analyze_sustainability(
        &self,
        cand: &ContractCandidate,
        reward: &RewardInfo,
    ) -> Result<SustainResult, ScanError> {
        // --- 必要条件检查 ---
        if reward.reward_tokens.is_empty() {
            return Ok(SustainResult { remaining_days: None });
        }

        let reward_rate = match reward.reward_rate {
            Some(v) if v > 0 => v as f64,
            _ => return Ok(SustainResult { remaining_days: None }),
        };

        let mut min_days: Option<f64> = None;

        for token in &reward.reward_tokens {
            // ERC20.balanceOf(staking_contract)
            let bal: Option<U256> = eth_call_view::<Uint<256>> (
                &self.client.provider(),
                *token,
                "function balanceOf(address) view returns (uint256)",
                &[DynSolValue::Address(cand.contract)],
            )
            .await;

            let bal = match bal {
                Some(v) if !v.is_zero() => v,
                _ => continue,
            };

            let remaining_seconds = bal.to::<u128>() as f64 / reward_rate;
            let days = remaining_seconds / 86400.0;

            min_days = match min_days {
                Some(cur) => Some(cur.min(days)),
                None => Some(days),
            };
        }

        Ok(SustainResult {
            remaining_days: min_days,
        })
    }

    fn evaluate_risk(
        &self,
        reward: &RewardInfo,
        sustain: &SustainResult,
    ) -> Vec<AprRiskFlag> {
        let mut out = vec![];

        if reward.modifiable {
            out.push(AprRiskFlag::OwnerModifiable);
        }

        if let Some(apr) = reward.apr {
            if apr > 300.0 {
                out.push(AprRiskFlag::VeryHighApr);
            }
        }

        out
    }
}

// ========================== funcs ==========================
fn abi_has_fn(abi: &JsonAbi, name: &str) -> bool {
    abi.functions.contains_key(name)
}

pub fn is_staking_rewards_abi(abi: &JsonAbi) -> bool {
    let mut hit = 0;

    if abi_has_fn(abi, "rewardRate") {
        hit += 1;
    }
    if abi_has_fn(abi, "periodFinish") {
        hit += 1;
    }
    if abi_has_fn(abi, "rewardsToken") {
        hit += 1;
    }
    if abi_has_fn(abi, "stakingToken") {
        hit += 1;
    }

    // 至少命中 2 个，认为是 StakingRewards
    hit >= 2
}

pub fn is_masterchef_abi(abi: &JsonAbi) -> bool {
    let mut hit = 0;

    if abi_has_fn(abi, "rewardPerBlock") {
        hit += 1;
    }
    if abi_has_fn(abi, "totalAllocPoint") {
        hit += 1;
    }
    if abi_has_fn(abi, "poolInfo") {
        hit += 1;
    }
    if abi_has_fn(abi, "poolLength") {
        hit += 1;
    }

    // 至少命中 2 个，认为是 MasterChef
    hit >= 2
}

async fn eth_call_view<T>(
    provider: &DynProvider,
    target: Address,
    func_sig: &str,
    args: &[DynSolValue],
) -> Option<T::RustType>
where
    T: SolType,
{
    let func = Function::parse(func_sig).ok()?;
    let calldata = func.abi_encode_input(args).ok()?;

    let tx = TransactionRequest {
        to: Some(target.into()),
        input: TransactionInput {
            input: Some(Bytes::copy_from_slice(&calldata)),
            data: None,
        },
        ..Default::default()
    };

    let raw = provider.call(tx).await.ok()?;
    T::abi_decode(&raw).ok()
}
