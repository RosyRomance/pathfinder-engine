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
    primitives::{Address, B256, FixedBytes, Bytes, LogData},
    rpc::types::{BlockId, BlockNumberOrTag, TransactionReceipt, Transaction, 
        BlockTransactions, TransactionInput, Filter, FilterBlockOption, 
        TransactionRequest, Log as AlloyLog},
    network::Ethereum,
};
use super::{
    evm::EvmClient,
    errors::ScanError,
    types::{BlockNumber, TxHash, TxReceipt, Log},
};

// balanceOf(address) selector
const BALANCE_OF_SELECTOR: [u8; 4] = [0x70, 0xa0, 0x82, 0x31];

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
                TxRef::Full(tx) => TxHash(*tx.inner.hash()),
                TxRef::Hash(h) => h,
            });
        }
        let _uniq: HashSet<_> = out.iter().clone().collect();
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
        from: u64,
        to: u64,
        address: Option<Address>,
        topics0: Option<[u8; 32]>,
    ) -> Result<Vec<Log>, ScanError> {
        let mut filter = Filter::new()
            .select(FilterBlockOption::Range {
                from_block: Some(BlockNumberOrTag::Number(from)),
                to_block: Some(BlockNumberOrTag::Number(to)),
            });

        if let Some(addr) = address {
            filter = filter.address(addr);
        }

        if let Some(t0) = topics0 {
            filter = filter.topic1(FixedBytes::from(t0));
        }

        let raw_logs = self.provider
            .get_logs(&filter)
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        let logs = raw_logs
            .into_iter()
            .map(convert_log)
            .collect();

        Ok(logs)
    }

    async fn get_code(
        &self,
        address: Address,
        _block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError> {
        let code = self.provider
            .get_code_at(address)
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        Ok(code.to_vec())
    }

    async fn call(
        &self,
        to: Address,
        data: Vec<u8>,
        _block: Option<BlockNumber>,
    ) -> Result<Vec<u8>, ScanError> {
        let tx = TransactionRequest {
            to: Some(alloy::primitives::TxKind::Call(to)),
            input: TransactionInput {
                input: Some(Bytes::copy_from_slice(&data)),
                data: None,
            },
            ..Default::default()
        };

        let raw = self.provider
            .call(tx)
            .await
            .map_err(|e| ScanError::Provider(e.to_string()))?;

        Ok(raw.to_vec())
    }

    async fn get_erc20_balance_of(
        &self,
        token: Address,
        owner: Address,
        block: Option<BlockNumber>,
    ) -> Result<[u8; 32], ScanError> {
        let mut data = [0u8; 36];
        data[..4].copy_from_slice(&BALANCE_OF_SELECTOR);
        data[4 + 12..].copy_from_slice(owner.as_slice());

        let raw = self.call(token, data.to_vec(), block).await?;

        if raw.len() < 32 {
            return Ok([0u8; 32]);
        }

        let mut out = [0u8; 32];
        out.copy_from_slice(&raw[raw.len() - 32..]);
        Ok(out)
    }

    fn provider(&self) -> & dyn Provider {
        &self.provider
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
                    .map(|h| TxRef::Hash(TxHash(h)))
            )
        }
        _ => Box::new(std::iter::empty()),
    }
}

fn convert_log(raw: AlloyLog<LogData>) -> Log {
    Log {
        address: raw.inner.address,
        topics: raw
            .topics()
            .iter()
            .map(|t| t.0)
            .collect(),
        data: raw.inner.data.data.as_ref().to_vec(),
        block_number: raw.block_number.unwrap_or_default(),
        tx_hash: TxHash::from_fixed(
            raw.transaction_hash.expect("log must have tx hash")),
    }
}