use alloy::{
    primitives::Address,
    providers::Provider,
};
use crate::{
    risk::{
        engine::SnapshotPoint,
        context::RiskStore,
    },
};
// ========================== Funcs ==========================

pub async fn collect_tvl<P: Provider>(
    provider: &P,
    chain_id: u64,
    contract: Address,
    block: u64,
) -> Result<f64, String> {
    // MVP: ETH balance
    let bal = provider
        .get_balance(contract)
        .await
        .map_err(|e| e.to_string())?;

    // convert to f64 (lossy is acceptable for risk)
    Ok(bal.to::<u128>() as f64)
}

// pub async fn collect_holder_snapshot<P: Provider>(
//     provider: &P,
//     chain_id: u64,
//     token: Address,
//     holders: &[Address],
//     block: u64,
// ) -> Result<Vec<(Address, f64, f64)>, String> {
//     let total = erc20_total_supply(provider, token).await?;

//     let mut out = Vec::new();
//     for h in holders {
//         let bal = erc20_balance_of(provider, token, *h).await?;
//         out.push((*h, bal, total));
//     }
//     Ok(out)
// }

// ========================== ClickhouseRiskStore ==========================

pub struct ClickHouseRiskStore {
    pub client: clickhouse::Client,
}

impl RiskStore for ClickHouseRiskStore {
    fn holder_concentration(
        &self,
        token: Address,
    ) -> Result<(f64, f64), String> {
        // query SQL in 3.5
        unimplemented!()
    }

    fn tvl_history_24h(
        &self,
        target: Address,
    ) -> Result<Vec<SnapshotPoint>, String> {
        // query SQL in 2.4
        unimplemented!()
    }

    fn contract_age_blocks(
        &self,
        target: Address,
        now_block: u64,
    ) -> Result<u64, String> {
        // deploy_block -> now_block - deploy
        unimplemented!()
    }
}


