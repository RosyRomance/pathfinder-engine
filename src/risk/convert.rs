use super::{
    model::RiskRecord,
    scorer::{RiskFlag, RiskReport},
};

// ========================== RiskContext ==========================

pub fn to_record(
    report: &RiskReport,
    scanned_block: u64,
    scanned_at_unix: u64,
) -> RiskRecord {
    let mut rec = RiskRecord {
        chain_id: report.chain_id,
        contract: report.target,
        scanned_block,
        scanned_at_unix,
        risk_score: report.score,

        has_owner_withdraw: false,
        has_pause: false,
        has_proxy_admin_eoa: false,
        has_high_concentration: false,
        has_high_tvl_volatility: false,

        top1_holder: None,
        top3_holder: None,
        tvl_change_24h: None,

        flags_json: String::new(),
    };

    for f in &report.flags {
        match f {
            RiskFlag::OwnerCanWithdraw => rec.has_owner_withdraw = true,
            RiskFlag::OwnerCanPause => rec.has_pause = true,
            RiskFlag::ProxyAdminIsEOA => rec.has_proxy_admin_eoa = true,

            RiskFlag::TVLConcentrationHigh { top1, top3 } => {
                rec.has_high_concentration = true;
                rec.top1_holder = Some(*top1);
                rec.top3_holder = Some(*top3);
            }

            RiskFlag::TVLVolatilityHigh { change_24h } => {
                rec.has_high_tvl_volatility = true;
                rec.tvl_change_24h = Some(*change_24h);
            }

            _ => {}
        }
    }

    // flags_json: MVP 用 debug 即可
    rec.flags_json = format!("{:?}", report.flags);

    rec
}
