use crate::{
    types::{Account, GasInformation},
    CRONCAT_NAME, CW20_NAME, RULES_NAME,
};
use anyhow::Result;
use cosm_orc::{
    config::{
        cfg::Coin,
        key::{Key, SigningKey},
    },
    orchestrator::cosm_orc::CosmOrc,
};
use cosmwasm_std::Binary;
use cw20::Cw20Coin;
use cw_croncat::contract::{GAS_ACTION_FEE_JUNO, GAS_BASE_FEE_JUNO, GAS_DENOMINATOR_DEFAULT_JUNO};
use cw_croncat_core::msg::{TaskRequest, TaskResponse};
use cw_rules_core::msg::RuleResponse;

pub(crate) fn init_contracts(
    orc: &mut CosmOrc,
    key: &SigningKey,
    admin_addr: &str,
    user_addr: &str,
    denom: impl Into<String>,
) -> Result<()> {
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), true)?;

    // TODO: split conctracts!
    // orc.store_contracts("artifacts", key, None)?;

    orc.contract_map.register_contract(
        CW20_NAME,
        std::env::var("CW20_ID").unwrap().parse().unwrap(),
    );
    orc.contract_map.register_contract(
        CRONCAT_NAME,
        std::env::var("CRONCAT_ID").unwrap().parse().unwrap(),
    );
    orc.contract_map.register_contract(
        RULES_NAME,
        std::env::var("RULES_ID").unwrap().parse().unwrap(),
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

    orc.instantiate(
        CRONCAT_NAME,
        "croncat_init",
        &croncat_msg,
        key,
        Some(admin_addr.to_owned()),
        vec![],
    )?;

    let cw20_msg = cw20_base::msg::InstantiateMsg {
        name: "Croncat".to_string(),
        symbol: "cct".to_string(),
        decimals: 3,
        initial_balances: vec![Cw20Coin {
            address: user_addr.to_owned(),
            amount: 100_000_u128.into(),
        }],
        mint: None,
        marketing: None,
    };

    orc.instantiate(
        CW20_NAME,
        "cw20_init",
        &cw20_msg,
        key,
        Some(admin_addr.to_owned()),
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

/// Create and complete tasks three times
///  
/// Last proxy call should unregister due to lack of balance
pub(crate) fn complete_tasks_for_three_times<S>(
    orc: &mut CosmOrc,
    (agent_key, agent_addr): (&SigningKey, S),
    user_key: &SigningKey,
    denom: S,
    tasks: Vec<(TaskRequest, u64, &str)>,
) -> Result<Vec<[GasInformation; 3]>>
where
    S: Into<String>,
{
    let denom = denom.into();
    let agent_addr = agent_addr.into();
    let base_attach =
        (GAS_BASE_FEE_JUNO + (GAS_BASE_FEE_JUNO * 5 / 100)) / GAS_DENOMINATOR_DEFAULT_JUNO;
    let attach_per_action =
        (GAS_ACTION_FEE_JUNO + (GAS_ACTION_FEE_JUNO * 5 / 100)) / GAS_DENOMINATOR_DEFAULT_JUNO;
    let prefixes: Vec<String> = tasks
        .iter()
        .map(|(_, _, prefix)| (*prefix).to_owned())
        .collect();
    for (task, extra_funds, prefix) in tasks {
        let amount =
            (base_attach + task.actions.len() as u64 * attach_per_action + extra_funds) * 3;
        let msg = cw_croncat_core::msg::ExecuteMsg::CreateTask { task };
        orc.execute(
            CRONCAT_NAME,
            &format!("{prefix}_create_task"),
            &msg,
            user_key,
            vec![Coin {
                denom: denom.clone(),
                amount,
            }],
        )?;
    }
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let first_proxys = proxy_call_for_n_times(
        orc,
        (agent_key, agent_addr.clone()),
        denom.clone(),
        prefixes
            .iter()
            .map(|prefix| format!("{prefix}_first_proxy_call"))
            .collect(),
    )?;
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let second_proxys = proxy_call_for_n_times(
        orc,
        (agent_key, agent_addr.clone()),
        denom.clone(),
        prefixes
            .iter()
            .map(|prefix| format!("{prefix}_middle_proxy_call"))
            .collect(),
    )?;

    // check that all of the tasks still active
    let query_res = orc.query(
        CRONCAT_NAME,
        &cw_croncat_core::msg::QueryMsg::GetTasks {
            from_index: None,
            limit: None,
        },
    )?;
    let tasks: Vec<TaskResponse> = query_res.data()?;
    if tasks.len() != tasks.len() {
        return Err(anyhow::anyhow!("{} tasks finihsed too early", tasks.len()));
    }

    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let third_proxys = proxy_call_for_n_times(
        orc,
        (agent_key, agent_addr),
        denom,
        prefixes
            .iter()
            .map(|prefix| format!("{prefix}_last_proxy_call"))
            .collect(),
    )?;
    // check that all of the tasks got unregistered
    let query_res = orc.query(
        CRONCAT_NAME,
        &cw_croncat_core::msg::QueryMsg::GetTasks {
            from_index: None,
            limit: None,
        },
    )?;
    let tasks: Vec<TaskResponse> = query_res.data()?;
    if !tasks.is_empty() {
        return Err(anyhow::anyhow!("{} tasks not finihsed", tasks.len()));
    }

    let res = first_proxys
        .into_iter()
        .zip(second_proxys.into_iter())
        .zip(third_proxys.into_iter())
        .map(|((one, two), three)| [one, two, three])
        .collect();
    Ok(res)
}

pub(crate) fn proxy_call_for_n_times(
    orc: &mut CosmOrc,
    (agent_key, agent_addr): (&SigningKey, String),
    denom: String,
    op_names: Vec<String>,
) -> Result<Vec<GasInformation>> {
    let mut gas_infos = Vec::with_capacity(op_names.len());
    for name in op_names.iter() {
        let before_pc = query_balance(orc, agent_addr.clone(), denom.clone())?;
        let res = orc.execute(
            CRONCAT_NAME,
            name,
            &cw_croncat_core::msg::ExecuteMsg::ProxyCall { task_hash: None },
            agent_key,
            vec![],
        )?;
        let after_pc = query_balance(orc, agent_addr.clone(), denom.clone())?;
        let gas_information = GasInformation {
            gas_used: res.res.gas_used,
            native_balance_burned: before_pc - after_pc,
        };
        gas_infos.push(gas_information);
    }
    Ok(gas_infos)
}

pub(crate) fn average_u64_slice(array: &[u64]) -> u64 {
    let sum: u64 = array.iter().sum();
    let count = array.len() as u64;
    sum / count
}

pub(crate) fn refill_cw20(orc: &mut CosmOrc, user_key: &SigningKey, amount: u128) -> Result<()> {
    let croncat_addr = orc.contract_map.address(CRONCAT_NAME)?;
    let msg = cw20_base::msg::ExecuteMsg::Send {
        contract: croncat_addr,
        amount: amount.into(),
        msg: Binary::default(),
    };
    orc.execute(CW20_NAME, "refill_cw20", &msg, user_key, vec![])?;
    Ok(())
}
