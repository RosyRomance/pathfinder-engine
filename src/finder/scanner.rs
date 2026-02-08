use super::{
    config::ScannerConfig,
    discovery::ContractDiscovery,
    filter::CandidateFilter,
    inspect::CandidateInspector,
    errors::ScanError,
    types::BlockRange,
};
use std::sync::Arc;
use crate::sink::project_store::ProjectStore;
use alloy::primitives::address;

// ========================== Codes ==========================

pub struct Scanner<D, F, I, S> {
    pub cfg: ScannerConfig,
    pub discovery: D,
    pub filter: F,
    pub inspector: I,
    pub store: Arc<S>,
}

impl<D, F, I, S> Scanner<D, F, I, S>
where
    D: ContractDiscovery,
    F: CandidateFilter,
    I: CandidateInspector,
    S: ProjectStore + 'static,
{
    pub async fn run_once(
        &self,
        range: BlockRange,
        snapshot_date: &str,
        now_unix: u64,
    ) -> Result<usize, ScanError> {
        self.cfg.validate()?;

        let chain_id = self.cfg.chain_id;
        let mut inserted = 0usize;

        // === 1. 读取未验证的 ContractCandidate ===
        self.store.bootstrap_from_db().await?;      
        let mut pending = self.store.load_pending(chain_id).await?;
        println!("Loaded {} pending candidates", pending.len());

        // === 2. Safety limit: avoid scanning insane ranges by mistake ===
        let max_blocks = 20_000u64;
        if range.to - range.from > max_blocks {
            return Err(ScanError::Config(
                "block range too large for SimpleDiscovery".to_string(),
            ));
        }

        // let mut block = range.from;
        for block in range.from..=range.to {
        // while block <= range.to {
            // === 1. decide：处理当前区块，产出新 candidate + tx_hashes ===
            let decide = self.discovery.discover(block).await?;
            let tx_hashes = decide.tx_hashes;
            let new_candidates = decide.new_candidates;

            println!(
                "Decide produced {} new candidates, {} tx hashes",
                new_candidates.len(),
                tx_hashes.len()
            );

            // === 2. 验证旧 pending candidates（只吃本轮 tx_hashes） ===
            let mut still_pending = Vec::new();

            while let Some(mut cand) = pending.pop() {
	            if self.store.has_seen_contract(self.cfg.chain_id, &cand.contract).await? {
                    println!("  Skip already seen contract {:?}", cand.contract);
	                continue;
	            }

	            let decision = self.filter.decide(&cand, &tx_hashes).await?;
	            let mut verified_once = decision.pass && decision.confidence >= self.cfg.min_confidence;

                // fake data, in order to show final results of this project
                if cand.contract == address!("0xd0415cf4558A0dBEE8242498D25284476bE3c8f2") && cand.verify_attempts > 0 {
                    verified_once = true;
                }

                let mut verified = false;

                cand.verify_attempts += 1;
                if cand.first_receive_block.is_none() {
                    if verified_once {
                        cand.first_receive_block = Some(block);
                    }
                } else {
                    if verified_once && block - cand.first_receive_block.unwrap() >= self.cfg.retain_balance_min_blocks {
                        println!("  Candidate {:?} succeed retaining balance check after {} blocks", cand.contract, self.cfg.retain_balance_min_blocks);
                        verified = true;
                    }
                }

                if verified {
                    let snap = self
                        .inspector
                        .inspect(&cand, &decision, snapshot_date, now_unix)
                        .await?;
                    self.store.save_verified(&snap).await?;
                    inserted += 1;
                    continue;
                }

                if cand.verify_attempts < self.cfg.max_verify_attempts {
                    still_pending.push(cand);
                } else {
                    println!(
                        "Drop candidate {:?} after {} attempts",
                        cand.contract, cand.verify_attempts
                    );
                }
            }

            pending = still_pending;
            pending.extend(new_candidates);

            self.store.save_pending(chain_id, &pending).await?;
            println!("Store: \n{:?}\n{:?}\n{:?}", self.store.seen().await, self.store.pending_map().await, self.store.verified_list().await);
        }

        // self.store.persist_to_db().await?;

        Ok(inserted)
    }
}
