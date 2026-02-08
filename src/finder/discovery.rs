use std::time::Instant;

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

        // 2) inspect receipts
        let receipts = self.client.fetch_receipts(&txs, 8).await;
        for (tx, receipt) in receipts {
            let mut interesting = false;

            // (A) contract creation → new candidate
            if let Some(addr) = receipt.contract_address {
                new_candidates.push(ContractCandidate {
                    chain_id: self.chain_id,
                    contract: addr,
                    deployed_block: receipt.block_number.unwrap() as u64,
                    verify_attempts: 0,
                    receives_token: None,
                    has_balance: None,
                    first_receive_block: None,
                    abi: None,
                });

                interesting = true;
            }

            // (B) logs hint token movement (用于后续验证)
            for lg in receipt.logs().iter() {
                if !lg.topics().is_empty() && lg.topics()[0] == TRANSFER_SIG {
                    interesting = true;
                    break;
                }
            }

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
