use cosm_orc::{
    config::{cfg::Coin, key::SigningKey},
    orchestrator::cosm_orc::CosmOrc,
};
use cosmwasm_std::{coin, coins, to_binary};
use cw20::Cw20Coin;
use cw_croncat_core::{
    msg::TaskRequest,
    types::{Action, Interval, Transform},
};
use cw_rules_core::types::Queries;
use generic_query::{GenericQuery, PathToValue, ValueIndex, ValueOrdering};

use crate::{helpers::query_balance, types::GasInformation, ALICE_ADDR, BOB_ADDR, CRONCAT_NAME};
use anyhow::Result;

pub(crate) fn complete_simple_task<S>(
    orc: &mut CosmOrc,
    (agent_key, agent_addr): (&SigningKey, S),
    user_key: &SigningKey,
    denom: S,
) -> Result<GasInformation>
where
    S: Into<String>,
{
    let denom = denom.into();
    let agent_addr = agent_addr.into();

    let task = cw_croncat_core::msg::TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: BOB_ADDR.to_owned(),
                amount: coins(1, denom.clone()),
            }),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    };
    let msg = cw_croncat_core::msg::ExecuteMsg::CreateTask { task };
    orc.execute(
        CRONCAT_NAME,
        "create_task",
        &msg,
        user_key,
        vec![Coin {
            denom: denom.clone(),
            amount: 1200000,
        }],
    )?;

    orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)?;
    let before_pc = query_balance(orc, agent_addr.clone(), denom.clone())?;
    let res = orc.execute(
        CRONCAT_NAME,
        "proxy_call",
        &cw_croncat_core::msg::ExecuteMsg::ProxyCall { task_hash: None },
        agent_key,
        vec![],
    )?;
    let after_pc = query_balance(orc, agent_addr, denom)?;

    let gas_information = GasInformation {
        gas_used: res.res.gas_used,
        native_balance_burned: before_pc - after_pc,
    };
    Ok(gas_information)
}

pub(crate) fn send_to_bob_recurring(denom: &str) -> TaskRequest {
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: cosmwasm_std::CosmosMsg::Bank(cosmwasm_std::BankMsg::Send {
                to_address: BOB_ADDR.to_owned(),
                amount: coins(1, denom),
            }),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

pub(crate) fn send_to_bob_and_alice_recurring(denom: &str) -> TaskRequest {
    let send_to_bob = Action {
        msg: cosmwasm_std::BankMsg::Send {
            to_address: BOB_ADDR.to_owned(),
            amount: coins(1, denom),
        }
        .into(),
        gas_limit: None,
    };
    let send_to_alice = Action {
        msg: cosmwasm_std::BankMsg::Send {
            to_address: ALICE_ADDR.to_owned(),
            amount: coins(2, denom),
        }
        .into(),
        gas_limit: None,
    };
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![send_to_bob, send_to_alice],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

pub(crate) fn send_cw20_to_bob_recurring(cw20_addr: &str, times: u128) -> TaskRequest {
    let amount_bob: u128 = 1;
    let amount = amount_bob;
    let msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: BOB_ADDR.to_owned(),
        amount: amount_bob.into(),
    };
    let send_cw20_to_bob = Action {
        msg: cosmwasm_std::WasmMsg::Execute {
            contract_addr: cw20_addr.to_owned(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        }
        .into(),
        gas_limit: Some(200_000),
    };
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![send_cw20_to_bob],
        queries: None,
        transforms: None,
        cw20_coins: vec![Cw20Coin {
            address: cw20_addr.to_owned(),
            amount: (times * amount).into(),
        }],
    }
}

pub(crate) fn send_cw20_to_bob_and_alice_recurring(cw20_addr: &str, times: u128) -> TaskRequest {
    let amount_bob: u128 = 1;
    let amount_alice: u128 = 2;
    let amount = amount_alice + amount_bob;
    let msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: BOB_ADDR.to_owned(),
        amount: amount_bob.into(),
    };
    let send_cw20_to_bob = Action {
        msg: cosmwasm_std::WasmMsg::Execute {
            contract_addr: cw20_addr.to_owned(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        }
        .into(),
        gas_limit: Some(200_000),
    };
    let msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: ALICE_ADDR.to_owned(),
        amount: amount_bob.into(),
    };
    let send_cw20_to_alice = Action {
        msg: cosmwasm_std::WasmMsg::Execute {
            contract_addr: cw20_addr.to_owned(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        }
        .into(),
        gas_limit: Some(200_000),
    };
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![send_cw20_to_bob, send_cw20_to_alice],
        queries: None,
        transforms: None,
        cw20_coins: vec![Cw20Coin {
            address: cw20_addr.to_owned(),
            amount: (times * amount).into(),
        }],
    }
}

pub(crate) fn delegate_to_bob_recurring(denom: &str) -> TaskRequest {
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![Action {
            msg: cosmwasm_std::CosmosMsg::Staking(cosmwasm_std::StakingMsg::Delegate {
                validator: BOB_ADDR.to_owned(),
                amount: coin(1, denom),
            }),
            gas_limit: None,
        }],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

pub(crate) fn delegate_to_bob_and_alice_recurring(denom: &str) -> TaskRequest {
    let delegate_to_bob = Action {
        msg: cosmwasm_std::CosmosMsg::Staking(cosmwasm_std::StakingMsg::Delegate {
            validator: BOB_ADDR.to_owned(),
            amount: coin(1, denom),
        }),
        gas_limit: None,
    };
    let delegate_to_alice = Action {
        msg: cosmwasm_std::CosmosMsg::Staking(cosmwasm_std::StakingMsg::Delegate {
            validator: ALICE_ADDR.to_owned(),
            amount: coin(3, denom),
        }),
        gas_limit: None,
    };
    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![delegate_to_bob, delegate_to_alice],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

pub(crate) fn delegate_to_validator(denom: &str) -> TaskRequest {
    let validator = std::env::var("VALIDATOR_ADDR").unwrap();
    let delegate_to_validator = Action {
        msg: cosmwasm_std::StakingMsg::Delegate {
            validator,
            amount: coin(1, denom),
        }
        .into(),
        gas_limit: None,
    };

    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![delegate_to_validator],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

pub(crate) fn delegate_to_validator_twice(denom: &str) -> TaskRequest {
    let validator = std::env::var("VALIDATOR_ADDR").unwrap();
    let delegate_to_validator = Action {
        msg: cosmwasm_std::StakingMsg::Delegate {
            validator,
            amount: coin(1, denom),
        }
        .into(),
        gas_limit: None,
    };

    TaskRequest {
        interval: Interval::Immediate,
        boundary: None,
        stop_on_fail: false,
        actions: vec![delegate_to_validator.clone(), delegate_to_validator],
        queries: None,
        transforms: None,
        cw20_coins: vec![],
    }
}

/// As soon as owner replaced - pay to the new one
pub(crate) fn send_cw20_to_insertable_addr(croncat_addr: &str, cw20_addr: &str, original_owner: &str) -> TaskRequest {
    let amount_placeholder: u128 = 10;
    let amount = amount_placeholder;
    let msg = cw20_base::msg::ExecuteMsg::Transfer {
        recipient: "croncat".to_owned(),
        amount: amount_placeholder.into(),
    };
    let send_cw20_to_placeholder = Action {
        msg: cosmwasm_std::WasmMsg::Execute {
            contract_addr: cw20_addr.to_owned(),
            msg: to_binary(&msg).unwrap(),
            funds: vec![],
        }
        .into(),
        gas_limit: Some(250_000),
    };
    let query = Queries::GenericQuery(GenericQuery {
        contract_addr: croncat_addr.to_string(),
        msg: to_binary(&cw_croncat::QueryMsg::GetConfig {}).unwrap(),
        path_to_value: vec![String::from("owner_id").into()].into(),
        ordering: ValueOrdering::NotEqual,
        value: to_binary(original_owner).unwrap(),
    });
    TaskRequest {
        interval: Interval::Once,
        boundary: None,
        stop_on_fail: false,
        actions: vec![send_cw20_to_placeholder],
        queries: Some(vec![query]),
        transforms: Some(vec![Transform {
            action_idx: 0,
            query_idx: 0,
            action_path: PathToValue(vec![
                ValueIndex::Key("transfer".to_owned()),
                ValueIndex::Key("recipient".to_owned()),
            ]),
            query_response_path: PathToValue(vec![]),
        }]),
        cw20_coins: vec![Cw20Coin {
            address: cw20_addr.to_owned(),
            amount: amount.into(),
        }],
    }
}
