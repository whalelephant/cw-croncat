use crate::{types::Account, CRONCAT_NAME, RULES_NAME};
use anyhow::Result;
use cosm_orc::{
    config::key::{Key, SigningKey},
    orchestrator::cosm_orc::CosmOrc,
};
use cosmwasm_std::Binary;
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
