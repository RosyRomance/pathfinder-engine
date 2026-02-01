use async_trait::async_trait;
use alloy::{
    primitives::{FixedBytes, Address, U256, TxKind, Bytes},
    rpc::types::{
        TransactionRequest, 
        TransactionInput,
        Log,
    },
};
use super::{
    types::{ContractCandidate, FilterDecision, FilterSignals, TxHash},
    errors::ScanError,
    evm::EvmClient,
    config::ScannerConfig,
};


// ERC20 Transfer(address,address,uint256)
const TRANSFER_SIG: FixedBytes<32> = FixedBytes([
    0xdd, 0xf2, 0x52, 0xad, 0x1b, 0xe2, 0xc8, 0x9b,
    0x69, 0xc2, 0xb0, 0x68, 0xfc, 0x37, 0x8d, 0xaa,
    0x95, 0x2b, 0xa7, 0xf1, 0x63, 0xc4, 0xa1, 0x16,
    0x28, 0xf5, 0x5a, 0x4d, 0xf5, 0x23, 0xb3, 0xef,
]);

const BALANCE_OF_SELECTOR: FixedBytes<4> = FixedBytes([
    0x70, 0xa0, 0x82, 0x31, // balanceOf(address)
]);


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

        let receives_token = self.check_receives_token(cand, &cand.tx_hashes).await?;
        let has_stake_like_methods = self.check_selectors(cand).await?;

        // retains_balance is hard without knowing token.
        // v1 strategy: if receives_token == true, pick top token(s) seen in Transfer logs and test balance retention.
        let retains_balance = if receives_token {
            self.check_retains_balance(cand, &cand.tx_hashes).await.unwrap_or(false)
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
        cand: &ContractCandidate,
        tx_hashes: &[TxHash],
    ) -> Result<bool, ScanError> {
        let target = cand.contract;

        for &h in tx_hashes {
            let receipt = match self.client.provider()
                .get_transaction_receipt(h.into())
                .await
            {
                Ok(Some(r)) => r,
                _ => continue,
            };

            for log in receipt.logs() {
                if is_erc20_receive(&log, target) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    async fn check_retains_balance(
        &self,
        cand: &ContractCandidate,
        tx_hashes: &[TxHash],
    ) -> Result<bool, ScanError> {
        let target = cand.contract;
        let mut seen_tokens = Vec::<Address>::new();

        // 1) 从 receipts 中找“接收过的 token”
        for &h in tx_hashes {
            let receipt = match self.client.provider()
                .get_transaction_receipt(h.into())
                .await
            {
                Ok(Some(r)) => r,
                _ => continue,
            };

            for log in receipt.logs() {
                if is_erc20_receive(&log, target) {
                    let token = log.inner.address;
                    if !seen_tokens.contains(&token) {
                        seen_tokens.push(token);
                    }
                }
            }
        }

        // 2) 对这些 token 调 balanceOf
        for token in seen_tokens {
            if self.balance_of(token, target).await? > U256::ZERO {
                return Ok(true);
            }
        }

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

    async fn balance_of(
        &self,
        token: Address,
        owner: Address,
    ) -> Result<U256, ScanError> {
        let mut data = [0u8; 36];
        data[..4].copy_from_slice(&BALANCE_OF_SELECTOR.0);
        data[4 + 12..].copy_from_slice(owner.as_slice());

        let tx = TransactionRequest {
            to: Some(TxKind::Call(token)),
            input: TransactionInput {
                input: Some(Bytes::copy_from_slice(&data)),
                data: None,
            },
            ..Default::default()
        };
        let raw = self.client
            .provider()
            .call(tx)
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        if raw.len() < 32 {
            return Ok(U256::ZERO);
        }

        Ok(U256::from_be_slice(&raw[raw.len() - 32..]))
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

fn is_erc20_receive(log: &Log, target: Address) -> bool {
    if log.topics().len() < 3 {
        return false;
    }
    if log.topics()[0] != TRANSFER_SIG {
        return false;
    }

    let to = Address::from_slice(&log.topics()[2].as_slice()[12..]);
    to == target
}