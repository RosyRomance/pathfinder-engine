use clickhouse::Client;
use crate::{
    finder::{
        errors::ScanError,
        types::ContractCandidate,
    },
};
use super::{
    types::{PendingContractRow, ProjectSnapshot, VerifiedProjectRow},
};

// ========================== Codes ==========================

#[derive(Clone)]
pub struct ClickhouseClient {
    pub client: Client,
}

impl ClickhouseClient {
    pub async fn query_pending_contracts(
        &self,
    ) -> Result<Vec<PendingContractRow>, ScanError> {
        let rows = self.client
            .query(
                r#"
                SELECT
                    chain_id,
                    contract,
                    deployed_block,
                    receives_token,
                    has_balance,
                    verify_attempts,
                    first_receive_block,
                    abi
                FROM pending_contracts
                "#,
            )
            .fetch_all::<PendingContractRow>()
            .await
            .map_err(|e| ScanError::Store(format!(
                "query pending_contracts failed: {e}"
            )))?;

        Ok(rows)
    }

    pub async fn query_verified_snapshots(
        &self,
    ) -> Result<Vec<ProjectSnapshot>, ScanError> {
        let rows = self.client
            .query(
                r#"
                SELECT
                    chain_id,
                    staking_contract,
                    snapshot_json
                FROM verified_projects
                "#,
            )
            .fetch_all::<VerifiedProjectRow>()
            .await
            .map_err(|e| ScanError::Store(format!(
                "query verified_projects failed: {e}"
            )))?;

        let mut out = Vec::with_capacity(rows.len());

        for row in rows {
            let snap: ProjectSnapshot = serde_json::from_str(&row.snapshot_json)
                .map_err(|e| ScanError::Store(format!(
                    "parse snapshot_json failed: {e}"
                )))?;
            out.push(snap);
        }

        Ok(out)
    }

    pub async fn insert_pending_contracts(
        &self,
        chain_id: u64,
        cands: &[ContractCandidate],
    ) -> Result<(), ScanError> {
        let mut insert = self.client
            .insert("pending_contracts")
            .map_err(|e| ScanError::Store(format!(
                "init insert pending_contracts failed: {e}"
            )))?;

        for c in cands {
            if c.chain_id as u64 != chain_id {
                continue;
            }

            let row = PendingContractRow::from_candidate(c)?;

            insert
                .write(&row)
                .await
                .map_err(|e| ScanError::Store(format!(
                    "insert pending_contracts row failed: {e}"
                )))?;
        }

        insert
            .end()
            .await
            .map_err(|e| ScanError::Store(format!(
                "finalize insert pending_contracts failed: {e}"
            )))?;

        Ok(())
    }

    pub async fn insert_verified_snapshots(
        &self,
        snaps: &[ProjectSnapshot],
    ) -> Result<(), ScanError> {
        let mut insert = self.client
            .insert("verified_projects")
            .map_err(|e| ScanError::Store(format!(
                "init insert verified_projects failed: {e}"
            )))?;

        for s in snaps {
            let row = VerifiedProjectRow {
                chain_id: s.chain_id as u64,
                staking_contract: format!("{:#x}", s.staking_contract),
                snapshot_json: serde_json::to_string(s)
                    .map_err(|e| ScanError::Store(format!(
                        "serialize snapshot failed: {e}"
                    )))?,
            };

            insert
                .write(&row)
                .await
                .map_err(|e| ScanError::Store(format!(
                    "insert verified_projects row failed: {e}"
                )))?;
        }

        insert
            .end()
            .await
            .map_err(|e| ScanError::Store(format!(
                "finalize insert verified_projects failed: {e}"
            )))?;

        Ok(())
    }
}
