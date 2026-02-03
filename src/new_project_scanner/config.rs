use serde::{Deserialize, Serialize};
use super::{
    types::ChainId,
    errors::ScanError,
};

// ========================== Codes ==========================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub chain_id: ChainId,
    pub min_confidence: f64,

    pub lookback_blocks: u64,
    pub max_candidates_per_run: usize,

    pub retain_balance_min_blocks: u64,
    pub stake_selectors_hex: Vec<String>, // 4-byte selector hex strings
    pub max_verify_attempts: u32,
}

impl ScannerConfig {
    pub fn validate(&self) -> Result<(), ScanError> {
        if !(0.0..=1.0).contains(&self.min_confidence) {
            return Err(ScanError::Config(
                "min_confidence must be in [0,1]".to_string(),
            ));
        }
        if self.lookback_blocks == 0 {
            return Err(ScanError::Config(
                "lookback_blocks must be > 0".to_string(),
            ));
        }
        Ok(())
    }
}