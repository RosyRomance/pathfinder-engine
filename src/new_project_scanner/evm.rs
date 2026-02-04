use async_trait::async_trait;
use alloy::{
    primitives::Address,
    providers::Provider,
    rpc::types::TransactionReceipt,
};
use super::{
    types::{BlockNumber, Log, TxHash, TxReceipt},
    errors::ScanError,
};


#[async_trait]
pub trait EvmClient: Send + Sync {
    async fn latest_block_number(&self) -> Result<BlockNumber, ScanError>;

    async fn get_logs(
        &self,
        from: BlockNumber,
        to: BlockNumber,
        address: Option<Address>,
        topics0: Option<[u8; 32]>,
    ) -> Result<Vec<Log>, ScanError>;

    async fn get_code(
        &self,
        address: Address,
        block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError>;

    async fn call(
        &self,
        to: Address,
        data: Vec<u8>,
        block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError>;

    async fn get_erc20_balance_of(
        &self,
        token: Address,
        owner: Address,
        block: Option<BlockNumber>,
    ) -> Result<[u8; 32], ScanError>;

    async fn get_block_tx_hashes(
        &self,
        block: BlockNumber,
    ) -> Result<Vec<TxHash>, ScanError>;

    async fn get_transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TxReceipt>, ScanError>;

    fn provider(&self) -> & dyn Provider;

    async fn fetch_receipts(&self, tx_hashes: &[TxHash], max_concurrency: usize, ) -> Vec<(TxHash, TransactionReceipt)>;
}

