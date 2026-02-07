-- 4.1 建表 SQL
CREATE TABLE pending_contracts (
    chain_id UInt64,
    contract String,
    deployed_block UInt64,
    receives_token Nullable(UInt8),
    has_balance Nullable(UInt8),
    verify_attempts UInt32,
    first_receive_block Nullable(UInt64),
    abi String
)
ENGINE = ReplacingMergeTree
ORDER BY (chain_id, contract);

CREATE TABLE verified_projects (
    chain_id UInt64,
    staking_contract String,
    snapshot_json String
)
ENGINE = ReplacingMergeTree
ORDER BY (chain_id, staking_contract);










CREATE TABLE IF NOT EXISTS risk_records
(
    chain_id UInt64,
    contract FixedString(20),

    scanned_block UInt64,
    scanned_at_unix UInt64,

    risk_score Float64,

    has_owner_withdraw UInt8,
    has_pause UInt8,
    has_proxy_admin_eoa UInt8,
    has_high_concentration UInt8,
    has_high_tvl_volatility UInt8,

    top1_holder Nullable(Float64),
    top3_holder Nullable(Float64),
    tvl_change_24h Nullable(Float64),

    flags_json String
)
ENGINE = ReplacingMergeTree(scanned_at_unix)
PARTITION BY chain_id
ORDER BY (chain_id, contract);

-- TVL Snapshot 表
CREATE TABLE IF NOT EXISTS tvl_snapshots
(
    chain_id UInt64,
    contract FixedString(20),

    block_number UInt64,
    ts_unix UInt64,

    tvl_raw Float64
)
ENGINE = MergeTree
PARTITION BY chain_id
ORDER BY (chain_id, contract, block_number);

-- 3.2 Holder Snapshot 表
CREATE TABLE IF NOT EXISTS holder_snapshots
(
    chain_id UInt64,
    token FixedString(20),
    holder FixedString(20),

    balance Float64,
    total_supply Float64,

    block_number UInt64,
    ts_unix UInt64
)
ENGINE = MergeTree
PARTITION BY chain_id
ORDER BY (chain_id, token, block_number);




-- 1. 找“高危新项目”
SELECT *
FROM risk_records
WHERE risk_score > 0.6
  AND has_owner_withdraw = 1
ORDER BY scanned_at_unix DESC;

-- 2. 找“巨鲸控制的质押池”
SELECT contract, top1_holder, top3_holder
FROM risk_records
WHERE has_high_concentration = 1
ORDER BY top1_holder DESC;

-- 3. 找“TVL 异动项目”
SELECT contract, tvl_change_24h
FROM risk_records
WHERE has_high_tvl_volatility = 1
ORDER BY abs(tvl_change_24h) DESC;

-- 2.4 RiskStore::tvl_history_24h 的 SQL
SELECT
    block_number,
    tvl_raw
FROM tvl_snapshots
WHERE chain_id = ?
  AND contract = ?
  AND ts_unix >= now() - 86400
ORDER BY block_number ASC;

-- 3.5 RiskStore::holder_concentration 的 SQL
