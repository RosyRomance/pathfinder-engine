use async_trait::async_trait;
use alloy::primitives::Address;
use tokio::sync::RwLock;
use std::collections::HashMap;
use super::{
    types::{ChainId, ProjectSnapshot, ContractCandidate},
    errors::ScanError,
};

// ========================== trait ==========================

// store 需要能读 / 写 pending
#[async_trait]
pub trait ProjectStore: Send + Sync {
    async fn upsert_snapshot(&self, snap: &ProjectSnapshot) -> Result<(), ScanError>;

    async fn has_seen_contract(&self, chain_id: ChainId, contract: &Address) -> Result<bool, ScanError>;

    async fn load_pending(&self, chain_id: u64) -> Result<Vec<ContractCandidate>, ScanError>;
    async fn save_pending(&self, chain_id: u64, cands: &[ContractCandidate]) -> Result<(), ScanError>;
    async fn save_verified(&self, snap: &ProjectSnapshot) -> Result<(), ScanError>;

    async fn seen(&self) -> Result<HashMap<String, bool>, ScanError>;
    async fn pending_map(&self) -> Result<HashMap<u64, Vec<ContractCandidate>>, ScanError>;
    async fn verified_list(&self) -> Result<Vec<ProjectSnapshot>, ScanError>;

    async fn save_risk_record(&self, record: &RiskRecord) -> Result<(), ScanError>;
}

// Minimal in-memory store for demo/testing (not persistent).
use std::collections::HashSet;
use std::sync::Mutex;

// ========================== Codes ==========================

pub struct MemoryStore {
    seen: Mutex<HashSet<String>>,
    /// chain_id -> pending candidates
    pending: RwLock<HashMap<u64, Vec<ContractCandidate>>>,

    /// verified snapshots (append-only is fine)
    verified: RwLock<Vec<ProjectSnapshot>>,
}

impl MemoryStore {
    pub fn new() -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            pending: RwLock::new(HashMap::new()),
            verified: RwLock::new(Vec::new()),
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

    async fn load_pending(
        &self,
        chain_id: u64,
    ) -> Result<Vec<ContractCandidate>, ScanError> {
        let guard = self.pending.read().await;

        Ok(guard
            .get(&chain_id)
            .cloned()
            .unwrap_or_else(Vec::new))
    }

    async fn save_pending(
        &self,
        chain_id: u64,
        cands: &[ContractCandidate],
    ) -> Result<(), ScanError> {
        let mut guard = self.pending.write().await;

        guard.insert(chain_id, cands.to_vec());

        Ok(())
    }

    async fn save_verified(
        &self,
        snap: &ProjectSnapshot,
    ) -> Result<(), ScanError> {
        let mut guard = self.verified.write().await;

        guard.push(snap.clone());

        Ok(())
    }

    async fn seen(&self) -> Result<HashMap<String, bool>, ScanError> {
        let g = self
            .seen
            .lock()
            .map_err(|_| ScanError::Store("lock poisoned".to_string()))?;
        let map = g.iter().map(|k| (k.clone(), true)).collect();
        Ok(map)
    }

    async fn pending_map(&self) -> Result<HashMap<u64, Vec<ContractCandidate>>, ScanError> {
        let guard = self.pending.read().await;
        Ok(guard.clone())
    }  

    async fn verified_list(&self) -> Result<Vec<ProjectSnapshot>, ScanError> {
        let guard = self.verified.read().await;
        Ok(guard.clone())
    }
}
