
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
        _range: BlockRange,
    ) -> Result<Vec<ContractCandidate>, ScanError> {
        // PSEUDOCODE:
        // for block in range:
        //   fetch block tx hashes
        //   for tx:
        //     receipt = getTransactionReceipt(tx)
        //     if receipt.contractAddress exists:
        //       push ContractCandidate
        //
        // NOTE: implement with your provider.
        Ok(Vec::new())
    }
}