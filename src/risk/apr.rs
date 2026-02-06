
const STAKING_REWARDS_ABI: &str = r#"
[
  {"name":"rewardRate","outputs":[{"type":"uint256"}],"stateMutability":"view","type":"function"},
  {"name":"periodFinish","outputs":[{"type":"uint256"}],"stateMutability":"view","type":"function"},
  {"name":"rewardsToken","outputs":[{"type":"address"}],"stateMutability":"view","type":"function"},
  {"name":"owner","outputs":[{"type":"address"}],"stateMutability":"view","type":"function"}
]
"#;

const MASTERCHEF_ABI: &str = r#"
[
  {"name":"rewardPerBlock","outputs":[{"type":"uint256"}],"stateMutability":"view","type":"function"},
  {"name":"totalAllocPoint","outputs":[{"type":"uint256"}],"stateMutability":"view","type":"function"},
  {"name":"poolInfo","inputs":[{"type":"uint256"}],"outputs":[
    {"name":"lpToken","type":"address"},
    {"name":"allocPoint","type":"uint256"}
  ],"stateMutability":"view","type":"function"},
  {"name":"owner","outputs":[{"type":"address"}],"stateMutability":"view","type":"function"}
]
"#;

const ERC20_BALANCE_ABI: &str = r#"
[
  {"name":"balanceOf","inputs":[{"type":"address"}],
   "outputs":[{"type":"uint256"}],
   "stateMutability":"view","type":"function"}
]
"#;

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

pub enum AprType {
    Inflationary,
    FeeSharing,
    Mixed,
    FixedPromise,   // 高危
    Unknown,
}

pub enum AprRiskFlag {
    ShortLivedRewards,
    OwnerCanModifyRewards,
    VeryHighApr,
    AprWithoutCap,
}

enum RewardModel {
    MasterChef { pid: u64 },
    StakingRewards,
    // Gauge,
    Unknown,
}

pub struct AprInspector {
    provider: Arc<dyn EvmProvider>,
}

impl AprInspector {
    pub async fn inspect(
        &self,
        cand: &ContractCandidate,
    ) -> Result<AprScanResult, ScanError> {
        // 1️⃣ 识别奖励模型
        let model = self.detect_reward_model(cand).await?;

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
                self.read_staking_rewards(cand.address).await
            }

            RewardModel::MasterChef { pid } => {
                self.read_masterchef_pool(cand.address, *pid).await
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
        let c = Contract::new(addr, STAKING_REWARDS_ABI, self.provider.clone());

        let reward_rate: Option<U256> =
            c.call("rewardRate", ()).await.ok();

        let period_finish: Option<U256> =
            c.call("periodFinish", ()).await.ok();

        let reward_token: Option<Address> =
            c.call("rewardsToken", ()).await.ok();

        let owner: Option<Address> =
            c.call("owner", ()).await.ok();

        // --- 可持续性估算 ---
        let remaining_days = if let (Some(rate), Some(finish)) =
            (reward_rate, period_finish)
        {
            let now = self.provider.block_timestamp().await.unwrap_or(0);
            let finish = finish.as_u64();

            if finish > now && !rate.is_zero() {
                Some(((finish - now) as f64) / 86400.0)
            } else {
                None
            }
        } else {
            None
        };

        Ok(RewardInfo {
            apr: None, // 不在这里算
            reward_rate: reward_rate.map(|v| v.as_u128()),
            reward_tokens: reward_token.into_iter().collect(),
            reward_remaining_days: remaining_days,
            modifiable: owner.is_some(), // 基本等价于可改
            owner,
        })
    }

    async fn read_masterchef_pool(
        &self,
        chef: Address,
        pid: u64,
    ) -> Result<RewardInfo, ScanError> {
        let c = Contract::new(chef, MASTERCHEF_ABI, self.provider.clone());

        let reward_per_block: Option<U256> =
            c.call("rewardPerBlock", ()).await.ok();

        let total_alloc: Option<U256> =
            c.call("totalAllocPoint", ()).await.ok();

        let pool: Option<(Address, U256)> =
            c.call("poolInfo", (pid,)).await.ok();

        let owner: Option<Address> =
            c.call("owner", ()).await.ok();

        let pool_reward = match (reward_per_block, total_alloc, pool.as_ref()) {
            (Some(rpb), Some(total), Some((_, alloc))) if !total.is_zero() => {
                Some((rpb * *alloc / total).as_u128())
            }
            _ => None,
        };

        Ok(RewardInfo {
            apr: None,
            reward_rate: pool_reward,      // per block
            reward_tokens: vec![],         // 通常是 protocol token
            reward_remaining_days: None,   // 需额外算奖励池余额
            modifiable: owner.is_some(),
            owner,
        })
    }

    pub async fn analyze_sustainability(
        &self,
        cand: &ContractCandidate,
        reward: &RewardInfo,
    ) -> Result<SustainResult, ScanError> {
        // 必要条件不足，直接返回 None
        if reward.reward_tokens.is_empty() {
            return Ok(SustainResult { remaining_days: None });
        }

        let reward_rate = match reward.reward_rate {
            Some(v) if v > 0 => v as f64,
            _ => return Ok(SustainResult { remaining_days: None }),
        };

        let mut min_days: Option<f64> = None;

        for token in &reward.reward_tokens {
            let bal = {
                let c = Contract::new(
                    *token,
                    ERC20_BALANCE_ABI,
                    self.provider.clone(),
                );
                match c.call::<_, U256>("balanceOf", (cand.address,)).await {
                    Ok(v) if !v.is_zero() => v,
                    _ => continue,
                }
            };

            let remaining_seconds = bal.as_u128() as f64 / reward_rate;
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
            out.push(AprRiskFlag::OwnerCanModifyRewards);
        }

        if let Some(apr) = reward.apr {
            if apr > 300.0 {
                out.push(AprRiskFlag::VeryHighApr);
            }
        }

        out
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



