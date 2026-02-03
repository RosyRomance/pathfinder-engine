use super::{
    config::ScannerConfig,
    discovery::ContractDiscovery,
    filter::CandidateFilter,
    inspect::CandidateInspector,
    store::ProjectStore,
    errors::ScanError,
    types::BlockRange,
};

// ========================== Codes ==========================

pub struct Scanner<D, F, I, S> {
    pub cfg: ScannerConfig,
    pub discovery: D,
    pub filter: F,
    pub inspector: I,
    pub store: S,
}

impl<D, F, I, S> Scanner<D, F, I, S>
where
    D: ContractDiscovery,
    F: CandidateFilter,
    I: CandidateInspector,
    S: ProjectStore,
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
        let mut pending = self.store.load_pending(chain_id).await?;
        println!("Loaded {} pending candidates", pending.len());

        // === 2. Safety limit: avoid scanning insane ranges by mistake ===
        let max_blocks = 20_000u64;
        if range.to - range.from > max_blocks {
            return Err(ScanError::Config(
                "block range too large for SimpleDiscovery".to_string(),
            ));
        }

        for block in range.from..=range.to {
            // === 1. decide：处理当前区块，产出新 candidate + tx_hashes ===
            let decide = self.discovery.discover(block).await?;
            let tx_hashes = decide.tx_hashes;
            let mut new_candidates = decide.new_candidates;

            // manage backpressure on new candidates
            // if new_candidates.len() > self.cfg.max_candidates_per_run {
            //     new_candidates.truncate(self.cfg.max_candidates_per_run);
            // }
            
            println!(
                "Decide produced {} new candidates, {} tx hashes",
                new_candidates.len(),
                tx_hashes.len()
            );

            // === 2. 验证旧 pending candidates（只吃本轮 tx_hashes） ===
            let mut still_pending = Vec::new();

            while let Some(cand) = pending.pop() {
	            if self.store.has_seen_contract(self.cfg.chain_id, &cand.contract).await? {
	                continue;
	            }
	            println!("store检查通过");

	            let decision = self.filter.decide(&cand, &tx_hashes).await?;
	            let verified = !decision.pass || decision.confidence < self.cfg.min_confidence;

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
	            println!("filter检查通过");
            }

            let mut pending = still_pending;
            for cand in new_candidates.into_iter() {
                pending.push(cand);
            }
            
            self.store.save_pending(chain_id, &pending).await?;
            println!("Pending candidates for next round: {}", pending.len());
        }

        Ok(inserted)
    }
}
