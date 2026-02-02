use async_trait ::async_trait;
use alloy::primitives::Address;
use super::{
    types::{ChainId, ProjectSnapshot},
    errors::ScanError,
};

// ========================== Codes ==========================

#[async_trait]
pub trait ProjectStore: Send + Sync {
    async fn upsert_snapshot(
        &self,
        snap: &ProjectSnapshot,
    ) -> Result<(), ScanError>;

    async fn has_seen_contract(
        &self,
        chain_id: ChainId,
        contract: &Address,
    ) -> Result<bool, ScanError>;
}

// Minimal in-memory store for demo/testing (not persistent).
use std::collections::HashSet;
use std::sync::Mutex;

pub struct MemoryStore {
    seen: Mutex<HashSet<String>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
        }
    }

    fn key(chain_id: ChainId, addr: &Address) -> String {
        let mut s = format!("{}:", chain_id);
        for b in addr.0 {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

#[async_trait]
impl ProjectStore for MemoryStore {
    async fn upsert_snapshot(
        &self,
        snap: &ProjectSnapshot,
    ) -> Result<(), ScanError> {
        let k = Self::key(snap.chain_id, &snap.staking_contract);
        let mut g = self
            .seen
            .lock()
            .map_err(|_| ScanError::Store("lock poisoned".to_string()))?;
        g.insert(k);
        Ok(())
    }

    async fn has_seen_contract(
        &self,
        chain_id: ChainId,
        contract: &Address,
    ) -> Result<bool, ScanError> {
        let k = Self::key(chain_id, contract);
        let g = self
            .seen
            .lock()
            .map_err(|_| ScanError::Store("lock poisoned".to_string()))?;
        Ok(g.contains(&k))
    }
}