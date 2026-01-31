
use async_trait::async_trait;
use super::{
    types::{ChainId, BlockRange, ContractCandidate},
    errors::ScanError,
    evm::EvmClient,
};


#[async_trait]
pub trait ContractDiscovery: Send + Sync {
    async fn discover(
        &self,
        range: BlockRange,
    ) -> Result<Vec<ContractCandidate>, ScanError>;
}

pub struct SimpleDiscovery<C: EvmClient> {
    pub chain_id: ChainId,
    pub client: C,
}

#[async_trait]
impl<C: EvmClient> ContractDiscovery for SimpleDiscovery<C> {
    async fn discover(
        &self,
        range: BlockRange,
    ) -> Result<Vec<ContractCandidate>, ScanError> {
        let mut out = Vec::new();

        // Safety limit: avoid scanning insane ranges by mistake
        let max_blocks = 20_000u64;
        if range.to - range.from > max_blocks {
            return Err(ScanError::Config(
                "block range too large for SimpleDiscovery".to_string(),
            ));
        }

        for block in range.from..=range.to {
            // 1) fetch all tx hashes in this block
            let txs = self.client.get_block_tx_hashes(block).await?;

            for tx in txs {
                // 2) fetch receipt
                let receipt = match self.client.get_transaction_receipt(tx).await? {
                    Some(r) => r,
                    None => continue,
                };

                // 3) contract creation tx?
                let Some(addr) = receipt.contract_address else {
                    continue;
                };

                out.push(ContractCandidate {
                    chain_id: self.chain_id,
                    contract: addr,
                    deployed_block: receipt.block_number,
                });
            }
        }

        Ok(out)
    }
}
