mod helpers;
mod types;

use anyhow::Result;
use cosm_orc::{
    config::cfg::{Coin, Config},
    orchestrator::cosm_orc::CosmOrc,
};
use cw_croncat_core::msg::TaskResponse;
use helpers::{execute_proxy, init_contracts, key_addr_from_account, register_agent};
use types::Account;

const RULES_NAME: &str = "cw_rules";
const CRONCAT_NAME: &str = "cw_croncat";
const DAODAO_VERSIONER_NAME: &str = "cw_daodao_versioner";
const REGISTRAR_NAME: &str = "cw_code_id_registry";

fn main() -> Result<()> {
    env_logger::init();

    let cfg = Config::from_yaml("ci/local_config.yaml")?;
    let denom = cfg.chain_cfg.denom.clone();
    let chain_id = cfg.chain_cfg.chain_id.clone();
    let mut orc = CosmOrc::new(cfg, true)?;
    let accounts: Vec<Account> = serde_json::from_slice(&std::fs::read("ci/test_accounts.json")?)?;
    let admin_account = accounts[0].clone();
    let agent_account = accounts[1].clone();
    let user_account = accounts[2].clone();

    let (admin_key, admin_addr) = key_addr_from_account(admin_account);
    let (agent_key, _agent_addr) = key_addr_from_account(agent_account);
    let (_user_key, _user_addr) = key_addr_from_account(user_account);

    init_contracts(&mut orc, &admin_key, &admin_addr, &denom)?;
    register_agent(&mut orc, &agent_key)?;

    orc.execute(
        REGISTRAR_NAME,
        "register_code_id",
        &cw_code_id_registry::msg::ExecuteMsg::Register {
            contract_name: "dao-contract".to_string(),
            version: "1".to_string(),
            chain_id: chain_id.clone(),
            code_id: orc.contract_map.code_id(CRONCAT_NAME)?,
            checksum: "todo".to_string(),
        },
        &admin_key,
        vec![],
    )
    .unwrap();

    orc.execute(
        DAODAO_VERSIONER_NAME,
        "create_versioner",
        &cw_daodao_versioner::msg::ExecuteMsg::CreateVersioner {
            daodao_addr: "todo".to_string(),
            name: "dao-contract".to_string(),
            chain_id,
        },
        &admin_key,
        vec![Coin {
            denom: denom.clone(),
            amount: 1_000_000_u64,
        }],
    )
    .unwrap();

    let res: Vec<TaskResponse> = orc
        .query(
            CRONCAT_NAME,
            &cw_croncat_core::msg::QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )?
        .data()?;
    println!("before proxy: {res:?}");

    // make sure task is created
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;

    execute_proxy(&mut orc, &agent_key)?;


    let res: Vec<TaskResponse> = orc
        .query(
            CRONCAT_NAME,
            &cw_croncat_core::msg::QueryMsg::GetTasks {
                from_index: None,
                limit: None,
            },
        )?
        .data()?;
    println!("after proxy: {res:?}");
    Ok(())
}
