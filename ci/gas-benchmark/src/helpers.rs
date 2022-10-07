use crate::{
    types::{Account, GasInformation},
    CRONCAT_NAME, RULES_NAME,
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
use cw_croncat::contract::{GAS_BASE_FEE_JUNO, GAS_FOR_ONE_NATIVE_JUNO};
use cw_croncat_core::msg::TaskRequest;
use cw_rules_core::msg::RuleResponse;

pub(crate) fn init_contracts(
    orc: &mut CosmOrc,
    key: &SigningKey,
    addr: &str,
    denom: impl Into<String>,
) -> Result<()> {
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), true)?;

    orc.store_contracts("artifacts", key, None)?;

    let rules_msg = cw_rules_core::msg::InstantiateMsg {};

    let rules_res = orc.instantiate(
        RULES_NAME,
        "rules_init",
        &rules_msg,
        key,
        Some(addr.to_owned()),
        vec![],
    )?;

    let croncat_msg = cw_croncat_core::msg::InstantiateMsg {
        denom: denom.into(),
        cw_rules_addr: rules_res.address,
        owner_id: Some(addr.to_owned()),
        gas_base_fee: None,
        agent_nomination_duration: None,
    };

    orc.instantiate(
        CRONCAT_NAME,
        "croncat_init",
        &croncat_msg,
        key,
        Some(addr.to_owned()),
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
    tasks: Vec<(TaskRequest, u64)>,
) -> Result<Vec<[GasInformation; 3]>>
where
    S: Into<String>,
{
    let denom = denom.into();
    let agent_addr = agent_addr.into();
    let attach_per_action = GAS_BASE_FEE_JUNO / GAS_FOR_ONE_NATIVE_JUNO;
    let num = tasks.len();
    for (task, extra_funds) in tasks {
        let amount = (task.actions.len() as u64 * attach_per_action + extra_funds) * 3;
        let msg = cw_croncat_core::msg::ExecuteMsg::CreateTask { task };
        orc.execute(
            CRONCAT_NAME,
            "create_task",
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
        num,
        "first_proxy_call",
    )?;
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let second_proxys = proxy_call_for_n_times(
        orc,
        (agent_key, agent_addr.clone()),
        denom.clone(),
        num,
        "middle_proxy_call",
    )?;
    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let third_proxys =
        proxy_call_for_n_times(orc, (agent_key, agent_addr), denom, num, "last_proxy_call")?;
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
    n: usize,
    op_name: &str,
) -> Result<Vec<GasInformation>> {
    let mut gas_infos = Vec::with_capacity(n);
    for _ in 0..n {
        let before_pc = query_balance(orc, agent_addr.clone(), denom.clone())?;
        let res = orc.execute(
            CRONCAT_NAME,
            op_name,
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

pub(crate) fn average_gas_for_one_native_ujunox(gas_fees_usage: Vec<GasInformation>) -> u64 {
    let (total_gas, total_ujunox) = gas_fees_usage
        .into_iter()
        .fold((0, 0), |(gas, ujunox), info| {
            (gas + info.gas_used, ujunox + info.native_balance_burned)
        });
    (total_gas as f64 / total_ujunox as f64).ceil() as u64
}

pub(crate) fn average_u64_slice(array: &[u64]) -> u64 {
    let sum: u64 = array.iter().sum();
    let count = array.len() as u64;
    sum / count
}
