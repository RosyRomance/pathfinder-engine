use async_trait::async_trait;
use futures::{stream, stream::FuturesUnordered, StreamExt};
use std::collections::HashSet;
use tokio::sync::Semaphore;
use std::sync::Arc;
use crate::risk::{
    scorer::RiskReport,
    engine::RiskContext,
};
use alloy::{
    hex,
    primitives::{FixedBytes, Address, U256, TxKind, Bytes, B256, b256},
    rpc::types::{
        TransactionRequest, 
        TransactionInput,
        TransactionReceipt,
        Log,
    },
    providers::Provider,
};
use super::{
    types::{ContractCandidate, FilterDecision, FilterSignals, TxHash},
    errors::ScanError,
    evm::EvmClient,
    config::ScannerConfig,
};

// ========================== Codes ==========================

// ERC20 Transfer(address,address,uint256)
pub const TRANSFER_SIG: B256 =
    b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
const BALANCE_OF_SELECTOR: FixedBytes<4> = FixedBytes([
    0x70, 0xa0, 0x82, 0x31, // balanceOf(address)
]);

// ========================== Codes ==========================

#[async_trait]
pub trait CandidateFilter: Send + Sync {
    async fn decide(
        &self,
        cand: &ContractCandidate,
        tx_hashes: &[TxHash],
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
        tx_hashes: &[TxHash],
    ) -> Result<FilterDecision, ScanError> {
        // Signals:
        // 1) receives_token: contract appears as `to` in ERC20 Transfer logs within a small range
        // 2) retains_balance: token balance stays > 0 after N blocks (requires choosing token)
        // 3) has_stake_like_methods: bytecode contains function selectors (rough heuristic)
        let receipts = self.client.fetch_receipts(tx_hashes, 8).await;

        let receives = self.check_receives_token_from_receipts(cand, &receipts);
        let has_stake_like_methods = self.check_selectors_score(cand).await?;

        // retains_balance is hard without knowing token.
        // v1 strategy: if receives_token == true, pick top token(s) seen in Transfer logs and test balance retention.
        let retains = if receives {
            self.check_retains_balance_from_receipts(cand, &receipts, 8)
                .await?
        } else {
            false
        };
        println!("retains: {}", retains);

        let mut pass_count = 0;
        if receives {
            pass_count += 1;
        }
        if retains {
            pass_count += 1;
        }
        if has_stake_like_methods > 0 {
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
                receives_token: receives,
                retains_balance: retains,
                has_stake_like_methods,
            },
        })
    }
}

impl<C: EvmClient> BehaviorFilter<C> {
    fn check_receives_token_from_receipts(
        &self,
        cand: &ContractCandidate,
        receipts: &[(TxHash, TransactionReceipt)],
    ) -> bool {
        let target = cand.contract;

        for (h, receipt) in receipts {
            println!(
                "  check_receives_token: tx={} has {} logs",
                h,
                receipt.logs().len()
            );

            for log in receipt.logs() {
                if is_erc20_receive(log, target) {
                    return true;
                }
            }
        }

        false
    }

    async fn check_retains_balance_from_receipts(
        &self,
        cand: &ContractCandidate,
        receipts: &[(TxHash, TransactionReceipt)],
        max_concurrency: usize,
    ) -> Result<bool, ScanError> {
        let target = cand.contract;
        let mut seen_tokens = HashSet::<Address>::new();

        for (h, receipt) in receipts {
            println!(
                "  check_retains_balance: tx={} has {} logs",
                h,
                receipt.logs().len()
            );

            for log in receipt.logs() {
                if is_erc20_receive(log, target) {
                    seen_tokens.insert(log.inner.address);
                }
            }
        }

        if seen_tokens.is_empty() {
            return Ok(false);
        }

        let target = target;

        let mut s = stream::iter(seen_tokens.into_iter())
            .map(|token| async move {
                let bal = self.balance_of(token, target).await?;
                Ok::<(Address, U256), ScanError>((token, bal))
            })
            .buffer_unordered(max_concurrency);

        while let Some(res) = s.next().await {
            let (token, bal) = res?;
            println!(
                "  check balance: token={} contract={} balance={}",
                token, target, bal
            );
            if bal > U256::ZERO {
                return Ok(true);
            }
        }

        Ok(false)
    }

    async fn check_selectors_score(
        &self,
        cand: &ContractCandidate,
    ) -> Result<u8, ScanError> {
        let code = self.client.get_code(cand.contract, Some(cand.deployed_block)).await?;
        if code.is_empty() {
            return Ok(0);
        }

        let blob: &[u8] = code.as_ref();
        let mut score: u8 = 0;

        for sel_hex in &self.cfg.stake_like_selectors {
            let sel = match hex4(sel_hex) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if blob.windows(4).any(|w| w == sel) {
                score = score.saturating_add(1);
            }
        }

        Ok(score)
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

    // pub async fn inspect_risk(
    //     &self,
    //     cand: &ContractCandidate,
    //     ctx: &dyn RiskContext,
    // ) -> Result<RiskReport, ScanError> {
    //     let engine = self.risk_engine(); // 你可以缓存在 self 里
    //     engine.run(ctx, cand.contract).map_err(ScanError::Config)
    // }
}

// ========================== Funcs ==========================

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