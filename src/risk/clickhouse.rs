use alloy::primitives::Address;
use crate::new_project_scanner::{
    errors::ScanError,
    store::ProjectStore,
};
use super::{
    engine::{RiskContext, SnapshotPoint},
    model::RiskRecord,
    context::RiskStore,
};

// ========================== RiskContext ==========================

pub struct ClickHouseStore {
    pub client: clickhouse::Client,
}

#[async_trait::async_trait]
impl ProjectStore for ClickHouseStore {
    async fn save_risk_record(
        &self,
        r: &RiskRecord,
    ) -> Result<(), ScanError> {
        self.client
            .insert("risk_records")
            .one((
                r.chain_id,
                r.contract.as_slice(),
                r.scanned_block,
                r.scanned_at_unix,
                r.risk_score,
                r.has_owner_withdraw as u8,
                r.has_pause as u8,
                r.has_proxy_admin_eoa as u8,
                r.has_high_concentration as u8,
                r.has_high_tvl_volatility as u8,
                r.top1_holder,
                r.top3_holder,
                r.tvl_change_24h,
                &r.flags_json,
            ))
            .await
            .map_err(|e| ScanError::Store(e.to_string()))
    }
}


// let report = engine.run(&ctx, cand.contract)?;

// let record = to_record(
//     &report,
//     now_block,
//     now_unix,
// );

// self.store.save_risk_record(&record).await?;


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
// 后续可扩展：
//     多 token
//     price feed
//     LP valuation
// RiskEngine 不需要改

// 3.4 Holder 采集代码骨架
pub async fn collect_holder_snapshot<P: Provider>(
    provider: &P,
    chain_id: u64,
    token: Address,
    holders: &[Address],
    block: u64,
) -> Result<Vec<(Address, f64, f64)>, String> {
    let total = erc20_total_supply(provider, token).await?;

    let mut out = Vec::new();
    for h in holders {
        let bal = erc20_balance_of(provider, token, *h).await?;
        out.push((*h, bal, total));
    }
    Ok(out)
}


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

