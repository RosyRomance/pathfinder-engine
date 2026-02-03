use async_trait::async_trait;
use super::{
    types::{ChainId, ContractCandidate, DecideOutput},
    errors::ScanError,
    evm::EvmClient,
    filter::TRANSFER_SIG,
};

// ========================== Codes ==========================

#[async_trait]
pub trait ContractDiscovery: Send + Sync {
    async fn discover(
        &self,
        range: u64,
    ) -> Result<DecideOutput, ScanError>;
}

pub struct SimpleDiscovery<C: EvmClient> {
    pub chain_id: ChainId,
    pub client: C,
}

#[async_trait]
impl<C: EvmClient> ContractDiscovery for SimpleDiscovery<C> {
    async fn discover(
        &self,
        block: u64,
    ) -> Result<DecideOutput, ScanError> {
        println!("Discovering contracts in block {}", block);
        let mut new_candidates = Vec::new();
        let mut tx_hashes = Vec::new();

        // 1) fetch tx hashes in this block
        let txs = self.client.get_block_tx_hashes(block).await?;
        if txs.is_empty() {
            return Ok(DecideOutput {
                new_candidates,
                tx_hashes,
            });
        }
        println!("  Found {} txs", txs.len());

        // 2) inspect receipts
        for tx in txs {
            let receipt = match self.client.get_transaction_receipt(tx).await? {
                Some(r) => r,
                None => continue,
            };

            let mut interesting = false;

            // (A) contract creation → new candidate
            if let Some(addr) = receipt.contract_address {
                new_candidates.push(ContractCandidate {
                    chain_id: self.chain_id,
                    contract: addr,
                    deployed_block: receipt.block_number,
                    verify_attempts: 0,
                    receives_token: None,
                    has_balance: None,
                });

                interesting = true;
            }
            println!("ContractCandidate check finished!");

            // (B) logs hint token movement (用于后续验证)
            let logs = self.client.get_logs(
                receipt.block_number.into(),
                receipt.block_number.into(),
                None,
                Some(TRANSFER_SIG.0),
            ).await?;
            for log in &logs {
                // 极简过滤：有 topic + 非空 data
                // 不在 decide 阶段做 ERC20 精确解析
                if !log.topics.is_empty() && !log.data.is_empty() {
                    interesting = true;
                    break;
                }
            }
            println!("Qualified TxHashes check finished!");

            if interesting {
                tx_hashes.push(tx);
            }
        }

        Ok(DecideOutput {
            new_candidates,
            tx_hashes,
        })
    }
}
