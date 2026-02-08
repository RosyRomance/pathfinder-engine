use async_trait::async_trait;
use std::sync::Arc;
use alloy::primitives::Address;
use tokio::sync::{RwLock, Mutex};
use std::collections::{HashMap, HashSet};
use crate::{
    finder::{
        types::{ChainId, ContractCandidate, ProjectProfile},
        errors::ScanError,
    },
    risk::model::{RiskRecord, RiskRecordRow},
    sink::{
        client::ClickhouseClient,
        types::ProjectSnapshot,
    },
};

// ========================== trait ==========================

// store 需要能读 / 写 pending
#[async_trait]
pub trait ProjectStore: Send + Sync {
    async fn upsert_snapshot(&self, snap: &ProjectProfile) -> Result<(), ScanError>;

    async fn has_seen_contract(&self, chain_id: ChainId, contract: &Address) -> Result<bool, ScanError>;

    async fn load_pending(&self, chain_id: u64) -> Result<Vec<ContractCandidate>, ScanError>;
    async fn save_pending(&self, chain_id: u64, cands: &[ContractCandidate]) -> Result<(), ScanError>;
    async fn save_verified(&self, snap: &ProjectProfile) -> Result<(), ScanError>;

    async fn seen(&self) -> Result<HashMap<String, bool>, ScanError>;
    async fn pending_map(&self) -> Result<HashMap<u64, Vec<ContractCandidate>>, ScanError>;
    async fn verified_list(&self) -> Result<Vec<ProjectProfile>, ScanError>;
    async fn snapshots_list(&self) -> Result<Vec<ProjectSnapshot>, ScanError>;

    async fn save_risk_record(&self, record: &RiskRecord) -> Result<(), ScanError>;

    async fn bootstrap_from_db(&self) -> Result<(), ScanError>;
    async fn persist_to_db(&self) -> Result<(), ScanError>;
}

// ========================== Codes ==========================

pub struct ClickhouseStore {
    seen: Mutex<HashSet<String>>,
    /// chain_id -> pending candidates
    pending: RwLock<HashMap<u64, Vec<ContractCandidate>>>,

    /// verified snapshots (append-only is fine)
    verified: RwLock<Vec<ProjectProfile>>,

    snapshots: RwLock<Vec<ProjectSnapshot>>,

    ch: Arc<ClickhouseClient>,
}

impl ClickhouseStore {
    pub fn new(ch: &ClickhouseClient) -> Self {
        Self {
            seen: Mutex::new(HashSet::new()),
            pending: RwLock::new(HashMap::new()),
            verified: RwLock::new(Vec::new()),
            snapshots: RwLock::new(Vec::new()),
            ch: Arc::new(ch.clone()),
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
impl ProjectStore for ClickhouseStore {
    async fn upsert_snapshot(
        &self,
        snap: &ProjectProfile,
    ) -> Result<(), ScanError> {
        let k = Self::key(snap.chain_id, &snap.staking_contract);
        let mut g = self
            .seen
            .lock()
            .await;
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
            .await;
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
        snap: &ProjectProfile,
    ) -> Result<(), ScanError> {
        let mut guard = self.verified.write().await;

        guard.push(snap.clone());

        Ok(())
    }

    async fn seen(&self) -> Result<HashMap<String, bool>, ScanError> {
        let g = self
            .seen
            .lock()
            .await;
        let map = g.iter().map(|k| (k.clone(), true)).collect();
        Ok(map)
    }

    async fn pending_map(&self) -> Result<HashMap<u64, Vec<ContractCandidate>>, ScanError> {
        let guard = self.pending.read().await;
        Ok(guard.clone())
    }  

    async fn verified_list(&self) -> Result<Vec<ProjectProfile>, ScanError> {
        let guard = self.verified.read().await;
        Ok(guard.clone())
    }

    async fn snapshots_list(&self) -> Result<Vec<ProjectSnapshot>, ScanError> {
        let guard = self.snapshots.read().await;
        Ok(guard.clone())
    }

    async fn bootstrap_from_db(&self) -> Result<(), ScanError> {
        // 1️⃣ 读取 pending
        let rows = self.ch.query_pending_contracts().await?;

        let mut pending_map: HashMap<u64, Vec<ContractCandidate>> = HashMap::new();

        for row in rows {
            pending_map
                .entry(row.chain_id)
                .or_default()
                .push(row.into_candidate()?);
        }

        {
            let mut guard = self.pending.write().await;
            *guard = pending_map;
        }

        // 2️⃣ 读取 verified
        let snaps = self.ch.query_verified_snapshots().await?;

        {
            let mut guard = self.snapshots.write().await;
            *guard = snaps;
        }

        // 3️⃣ 填充 seen（避免重复扫）
        {
            let mut seen = self
                .seen
                .lock()
                .await;

            for snap in self.snapshots.read().await.iter() {
                let k = Self::key(snap.chain_id, &snap.staking_contract);
                seen.insert(k);
            }
        }

        Ok(())
    }

    async fn persist_to_db(&self) -> Result<(), ScanError> {
        // 1️⃣ flush pending
        let pending = self.pending.read().await;

        self.ch.truncate_pending_contracts().await?;
        for (chain_id, cands) in pending.iter() {
            self.ch
                .insert_pending_contracts(*chain_id, cands)
                .await?;
        }

        // 2️⃣ flush verified
        let snapshots = self.snapshots.read().await;

        self.ch.truncate_verified_snapshots().await?;
        self.ch
            .insert_verified_snapshots(&snapshots)
            .await?;

        Ok(())
    }

    async fn save_risk_record(
        &self,
        r: &RiskRecord,
    ) -> Result<(), ScanError> {
        let mut insert = self.ch
            .client
            .insert("risk_records")
            .map_err(|e| ScanError::Store(e.to_string()))?;

        let row = RiskRecordRow {
            chain_id: r.chain_id,
            contract: r.contract.as_slice().to_vec(),
            scanned_block: r.scanned_block,
            scanned_at_unix: r.scanned_at_unix,
            risk_score: r.risk_score,

            has_owner_withdraw: r.has_owner_withdraw as u8,
            has_pause: r.has_pause as u8,
            has_proxy_admin_eoa: r.has_proxy_admin_eoa as u8,
            has_high_concentration: r.has_high_concentration as u8,
            has_high_tvl_volatility: r.has_high_tvl_volatility as u8,

            top1_holder: r.top1_holder,
            top3_holder: r.top3_holder,
            tvl_change_24h: r.tvl_change_24h,

            flags_json: r.flags_json.clone(),
        };

        insert
            .write(&row)
            .await
            .map_err(|e| ScanError::Store(e.to_string()))?;

        insert
            .end()
            .await
            .map_err(|e| ScanError::Store(e.to_string()))?;

        Ok(())
    }
}
