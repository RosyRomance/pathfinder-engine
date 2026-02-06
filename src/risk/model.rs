use alloy::primitives::Address;

// ========================== RiskContext ==========================

#[derive(Debug, Clone)]
pub struct RiskRecord {
    pub chain_id: u64,
    pub contract: Address,

    // 扫描时间
    pub scanned_block: u64,
    pub scanned_at_unix: u64,

    // 汇总
    pub risk_score: f64,

    // 冗余字段，方便 SQL 过滤
    pub has_owner_withdraw: bool,
    pub has_pause: bool,
    pub has_proxy_admin_eoa: bool,
    pub has_high_concentration: bool,
    pub has_high_tvl_volatility: bool,

    // 数值型风险
    pub top1_holder: Option<f64>,
    pub top3_holder: Option<f64>,
    pub tvl_change_24h: Option<f64>,

    // 原始 flags，给前端 / CLI 用
    pub flags_json: String,
}
