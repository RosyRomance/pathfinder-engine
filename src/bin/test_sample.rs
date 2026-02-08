use anyhow::Result;
use clickhouse::Client;
use alloy::primitives::address;
use pathfinder::{
    finder::{
        self, 
        alloy_evm::AlloyEvmClient, 
        types::{BlockRange, ContractCandidate},
    }, 
    sink::{
        client::ClickhouseClient, project_store::ClickhouseStore, 
        risk_store::ClickHouseRiskStore,
    },
};

// ===================== Demo main =====================

#[tokio::main]
async fn main() -> Result<()> {
    let cond = ContractCandidate {
        chain_id: 1,
        contract: address!("0xd0415cf4558A0dBEE8242498D25284476bE3c8f2"),
        deployed_block: 0,
        receives_token: None,
        has_balance: None,
        verify_attempts: 0,
        first_receive_block: None,
        abi: None,
    };
    let ch_client = ClickhouseClient { 
        client:  Client::default()
            .with_url("http://127.0.0.1:8123")
            .with_user("default")
            .with_password("StrongerPwd456!")
            .with_database("pathfinder")
        };
    ch_client.truncate_table("pending_contracts").await?;
    ch_client.insert_pending_contracts(1, &[cond]).await?;

    Ok(())
}