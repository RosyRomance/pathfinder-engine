use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashSet;
use url::{Url, ParseError};
use alloy::{
    providers::{
    Provider,
    RootProvider,
    Identity,
    ProviderBuilder,
    fillers::{
        FillProvider,
        JoinFill,
        GasFiller,
        BlobGasFiller,
        NonceFiller,
        ChainIdFiller,
        },
    },
    primitives::{Address, B256 },
    rpc::types::{BlockId, BlockNumberOrTag, TransactionReceipt, Transaction, BlockTransactions },
    network::Ethereum,
};
use super::{
    evm::EvmClient,
    errors::ScanError,
    types::{BlockNumber, TxHash, TxReceipt, Log},
};


// ========================== pub types ==========================

pub type DefaultFiller = JoinFill<
    Identity,
    JoinFill<
        GasFiller,
        JoinFill<
            BlobGasFiller,
            JoinFill<NonceFiller, ChainIdFiller>,
        >,
    >,
>;

/// FillProvider<F, P, N>
pub type DefaultProvider = FillProvider<
    DefaultFiller,
    RootProvider<Ethereum>,  // base provider
    Ethereum                      // network
>;

pub type SharedProvider = Arc<DefaultProvider>;

// ========================== Codes ==========================

#[derive(Clone, Debug)]
pub struct AlloyEvmClient {
    provider: DefaultProvider,
}

impl AlloyEvmClient {
    pub fn new_client(rpc_url: &str) -> Result<Self, ScanError> {
        let url: Url = rpc_url.parse().map_err(|e: ParseError| ScanError::Config(e.to_string()))?;

        let provider = ProviderBuilder::new()
            .connect_http(url);

        Ok(Self { provider })
    }
}

#[async_trait]
impl EvmClient for AlloyEvmClient {
    async fn latest_block_number(&self) -> Result<BlockNumber, ScanError> {
        let num = self.provider
            .get_block_number()
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        Ok(num)
    }

    async fn get_block_tx_hashes(
        &self,
        block: BlockNumber,
    ) -> Result<Vec<TxHash>, ScanError> {
        let block_id = BlockId::Number(BlockNumberOrTag::Number(block.into()));

        let block_opt = self.provider
            .get_block(block_id)
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        let block = match block_opt {
            Some(b) => b,
            None => return Ok(Vec::new()),
        };

        let mut out = Vec::with_capacity(block.transactions.len());
        for tx_ref in iter_txs(&block.transactions) {
            out.push(match tx_ref {
                TxRef::Full(tx) => TxHash(**tx.inner.hash()),
                TxRef::Hash(h) => h,
            });
        }
        let uniq: HashSet<_> = out.iter().clone().collect();
        // println!("tx hashes: total={}, unique={}", out.len(), uniq.len());

        Ok(out)
    }

    async fn get_transaction_receipt(
        &self,
        tx: TxHash,
    ) -> Result<Option<TxReceipt>, ScanError> {
        let receipt_opt: Option<TransactionReceipt> = self.provider
            .get_transaction_receipt(B256::from(tx.0))
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        let Some(r) = receipt_opt else {
            return Ok(None);
        };

        Ok(Some(TxReceipt {
            contract_address: r.contract_address.map(|a| Address(a.0)),
            block_number: r.block_number.unwrap_or_default(),
        }))
    }

    async fn get_logs(
        &self,
        from: BlockNumber,
        to: BlockNumber,
        address: Option<Address>,
        topics0: Option<[u8; 32]>,
    ) -> Result<Vec<Log>, ScanError>{
        let log: Vec<Log> = Vec::new();
        Ok(log)
    }

    async fn get_code(
        &self,
        address: Address,
        block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError>{
        let code: Vec<u8> = Vec::new();
        Ok(code)
    }

    async fn call(
        &self,
        to: Address,
        data: Vec<u8>,
        block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError>{
        let res: Vec<u8> = Vec::new();
        Ok(res)
    }

    async fn get_erc20_balance_of(
        &self,
        token: Address,
        owner: Address,
        block: Option<BlockNumber>,
    ) -> Result<[u8; 32], ScanError>{
        let balance: [u8; 32] = [0u8; 32];
        Ok(balance)
    }
}

// ========================== Funcs ==========================

pub enum TxRef<'a> {
    Full(&'a Transaction),
    Hash(TxHash),
}

fn iter_txs<'a>(
    txs: &'a BlockTransactions<Transaction>,
) -> Box<dyn Iterator<Item = TxRef<'a>> + 'a> {
    match txs {
        BlockTransactions::Full(v) => {
            Box::new(v.iter().map(TxRef::Full))
        }
        BlockTransactions::Hashes(hashes) => {
            Box::new(
                hashes
                    .iter()
                    .copied()
                    .map(|h| TxRef::Hash(TxHash(*h)))
            )
        }
        _ => Box::new(std::iter::empty()),
    }
}
