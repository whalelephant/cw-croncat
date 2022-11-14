use crate::{types::Account, CRONCAT_NAME, DAODAO_VERSIONER_NAME, REGISTRAR_NAME, RULES_NAME};
use anyhow::Result;
use cosm_orc::{
    client::chain_res::ExecResponse,
    config::{
        cfg::Coin,
        key::{Key, SigningKey},
    },
    orchestrator::cosm_orc::CosmOrc,
};
use cosmwasm_std::Binary;
use cw_croncat::ExecuteMsg;
use cw_croncat_core::msg::TaskRequest;
use cw_rules_core::msg::RuleResponse;

pub(crate) fn init_contracts(
    orc: &mut CosmOrc,
    key: &SigningKey,
    admin_addr: &str,
    denom: impl Into<String>,
) -> Result<()> {
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), true)?;

    // TODO: split conctracts!
    // orc.store_contracts("artifacts", key, None)?;
    // orc.store_contracts("ci/artifacts", key, None)?;

    orc.contract_map.register_contract(
        REGISTRAR_NAME,
        std::env::var("REGISTRAR_ID").unwrap().parse().unwrap(),
    );
    orc.contract_map.register_contract(
        CRONCAT_NAME,
        std::env::var("CRONCAT_ID").unwrap().parse().unwrap(),
    );
    orc.contract_map.register_contract(
        RULES_NAME,
        std::env::var("RULES_ID").unwrap().parse().unwrap(),
    );
    orc.contract_map.register_contract(
        DAODAO_VERSIONER_NAME,
        std::env::var("VERSIONER_ID").unwrap().parse().unwrap(),
    );

    let rules_msg = cw_rules_core::msg::InstantiateMsg {};

    let rules_res = orc.instantiate(
        RULES_NAME,
        "rules_init",
        &rules_msg,
        key,
        Some(admin_addr.to_owned()),
        vec![],
    )?;

    let croncat_msg = cw_croncat_core::msg::InstantiateMsg {
        denom: denom.into(),
        cw_rules_addr: rules_res.address,
        owner_id: Some(admin_addr.to_owned()),
        gas_action_fee: None,
        gas_fraction: None,
        agent_nomination_duration: None,
        gas_base_fee: None,
    };

    let croncat_res = orc.instantiate(
        CRONCAT_NAME,
        "croncat_init",
        &croncat_msg,
        key,
        Some(admin_addr.to_owned()),
        vec![],
    )?;
    let registrar_res = orc.instantiate(
        REGISTRAR_NAME,
        "registrar_init",
        &cw_code_id_registry::msg::InstantiateMsg {
            admin: admin_addr.to_owned(),
        },
        key,
        None,
        vec![],
    )?;

    orc.instantiate(
        DAODAO_VERSIONER_NAME,
        "versioner_init",
        &cw_daodao_versioner::msg::InstantiateMsg {
            registrar_addr: registrar_res.address,
            croncat_addr: croncat_res.address,
        },
        key,
        None,
        vec![],
    )?;

    Ok(())
}

pub(crate) fn key_addr_from_account(
    Account {
        name,
        address,
        mnemonic,
    }: Account,
) -> (SigningKey, String) {
    (
        SigningKey {
            name,
            key: Key::Mnemonic(mnemonic),
        },
        address,
    )
}

pub(crate) fn register_agent(orc: &mut CosmOrc, key: &SigningKey) -> Result<()> {
    let msg = cw_croncat_core::msg::ExecuteMsg::RegisterAgent {
        payable_account_id: None,
    };
    orc.execute(CRONCAT_NAME, "register_agent", &msg, key, vec![])?;
    Ok(())
}

#[allow(unused)]
pub(crate) fn query_balance<S>(orc: &mut CosmOrc, addr: S, denom: S) -> Result<u128>
where
    S: Into<String>,
{
    let res = orc.query(
        RULES_NAME,
        &cw_rules_core::msg::QueryMsg::GetBalance {
            address: addr.into(),
            denom: denom.into(),
        },
    )?;
    let res_bin = res
        .res
        .data
        .ok_or_else(|| anyhow::anyhow!("No result from query_balance"))?;
    let query_res: RuleResponse<Option<Binary>> = serde_json::from_slice(&res_bin)?;
    let balance_bin: Binary = query_res
        .1
        .ok_or_else(|| anyhow::anyhow!("No balance from query"))?;
    let balance: cosmwasm_std::Coin = serde_json::from_slice(balance_bin.as_slice())?;

    Ok(balance.amount.u128())
}

#[allow(unused)]
pub(crate) fn create_task(
    orc: &mut CosmOrc,
    task_request: TaskRequest,
    signer: &SigningKey,
    funds: Vec<Coin>,
) -> Result<ExecResponse> {
    let res = orc.execute(
        CRONCAT_NAME,
        "create_task",
        &ExecuteMsg::CreateTask { task: task_request },
        signer,
        funds,
    )?;
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), true)?;
    Ok(res)
}

pub(crate) fn execute_proxy(orc: &mut CosmOrc, agent_key: &SigningKey) -> Result<ExecResponse> {
    let res = orc.execute(
        CRONCAT_NAME,
        "proxy_call",
        &ExecuteMsg::ProxyCall { task_hash: None },
        agent_key,
        vec![],
    )?;
    Ok(res)
}
