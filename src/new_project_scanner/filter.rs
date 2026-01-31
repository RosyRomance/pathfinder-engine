use async_trait::async_trait;
use super::{
    types::{ContractCandidate, FilterDecision, FilterSignals},
    errors::ScanError,
    evm::EvmClient,
    config::ScannerConfig,
};


#[async_trait]
pub trait CandidateFilter: Send + Sync {
    async fn decide(
        &self,
        cand: &ContractCandidate,
    ) -> Result<FilterDecision, ScanError>;
}

pub struct BehaviorFilter<C: EvmClient> {
    pub cfg: ScannerConfig,
    pub client: C,
}

#[async_trait]
impl<C: EvmClient> CandidateFilter for BehaviorFilter<C> {
    async fn decide(
        &self,
        cand: &ContractCandidate,
    ) -> Result<FilterDecision, ScanError> {
        // Signals:
        // 1) receives_token: contract appears as `to` in ERC20 Transfer logs within a small range
        // 2) retains_balance: token balance stays > 0 after N blocks (requires choosing token)
        // 3) has_stake_like_methods: bytecode contains function selectors (rough heuristic)

        let receives_token = self.check_receives_token(cand).await?;
        let has_stake_like_methods = self.check_selectors(cand).await?;

        // retains_balance is hard without knowing token.
        // v1 strategy: if receives_token == true, pick top token(s) seen in Transfer logs and test balance retention.
        let retains_balance = if receives_token {
            self.check_retains_balance(cand).await.unwrap_or(false)
        } else {
            false
        };

        let mut pass_count = 0;
        if receives_token {
            pass_count += 1;
        }
        if retains_balance {
            pass_count += 1;
        }
        if has_stake_like_methods {
            pass_count += 1;
        }

        let pass = pass_count >= 2;

        let confidence = match pass_count {
            3 => 0.90,
            2 => 0.70,
            1 => 0.30,
            _ => 0.05,
        };

        Ok(FilterDecision {
            pass,
            confidence,
            signals: FilterSignals {
                receives_token,
                retains_balance,
                has_stake_like_methods,
            },
        })
    }
}

impl<C: EvmClient> BehaviorFilter<C> {
    async fn check_receives_token(
        &self,
        _cand: &ContractCandidate,
    ) -> Result<bool, ScanError> {
        // PSEUDOCODE:
        // topic0 = keccak256("Transfer(address,address,uint256)")
        // logs = get_logs(from, to, address=None, topics0=topic0)
        // check if any log has `to` == cand.contract in indexed topic1/topic2
        Ok(false)
    }

    async fn check_retains_balance(
        &self,
        _cand: &ContractCandidate,
    ) -> Result<bool, ScanError> {
        // PSEUDOCODE:
        // identify candidate tokens from transfer logs where to==contract
        // for token in top_tokens:
        //   bal0 = balanceOf(token, contract, block=deployed_block+1)
        //   bal1 = balanceOf(token, contract, block=deployed_block+retain_balance_min_blocks)
        //   if bal0 > 0 and bal1 > 0: return true
        Ok(false)
    }

    async fn check_selectors(
        &self,
        cand: &ContractCandidate,
    ) -> Result<bool, ScanError> {
        let code = self.client.get_code(cand.contract.clone(), Some(cand.deployed_block)).await?;
        if code.is_empty() {
            return Ok(false);
        }

        // Very rough heuristic:
        // - treat code bytes as blob and search for 4-byte selectors
        // - not reliable but cheap.
        let blob = code;
        for sel_hex in &self.cfg.stake_selectors_hex {
            if let Ok(sel) = hex4(sel_hex) {
                if blob.windows(4).any(|w| w == sel) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}

fn hex4(s: &str) -> Result<[u8; 4], ScanError> {
    // Expect "0x" optional, then 8 hex chars
    let t = s.strip_prefix("0x").unwrap_or(s);
    if t.len() != 8 {
        return Err(ScanError::Config("selector hex must be 8 chars".to_string()));
    }
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] = u8::from_str_radix(&t[i * 2..i * 2 + 2], 16)
            .map_err(|_| ScanError::Config("bad hex".to_string()))?;
    }
    Ok(out)
}