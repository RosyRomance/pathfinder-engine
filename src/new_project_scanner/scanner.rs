use super::{
    config::ScannerConfig,
    discovery::ContractDiscovery,
    filter::CandidateFilter,
    inspect::CandidateInspector,
    store::ProjectStore,
    errors::ScanError,
    types::BlockRange,
};


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

        let mut inserted = 0usize;

        let mut cands = self.discovery.discover(range).await?;
        if cands.len() > self.cfg.max_candidates_per_run {
            cands.truncate(self.cfg.max_candidates_per_run);
        }

        for cand in cands {
            if self.store.has_seen_contract(self.cfg.chain_id, &cand.contract).await? {
                continue;
            }

            let decision = self.filter.decide(&cand).await?;
            if !decision.pass || decision.confidence < self.cfg.min_confidence {
                continue;
            }

            let snap = self
                .inspector
                .inspect(&cand, &decision, snapshot_date, now_unix)
                .await?;

            self.store.upsert_snapshot(&snap).await?;
            inserted += 1;
        }

        Ok(inserted)
    }
}