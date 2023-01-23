use crate::msg::*;
use crate::{
    contract::instantiate,
    error::ContractError,
    state::{DEFAULT_MIN_TASKS_PER_AGENT, DEFAULT_NOMINATION_DURATION},
};
use cosmwasm_std::{
    coins,
    testing::{mock_env, mock_info},
    Addr, DepsMut, Empty, Response,
};

pub const AGENT0: &str = "agent0a7uhnpqthunr2rzj0ww0hwurpn42wyun6c5puz";
pub const AGENT1: &str = "agent17muvdgkep4ndptnyg38eufxsssq8jr3wnkysy8";
pub const AGENT2: &str = "agent2qxywje86amll9ptzxmla5ah52uvsd9f7drs2dl";
pub const AGENT3: &str = "agent3c3cy3wzzz3698ypklvh7shksvmefj69xhm89z2";
pub const AGENT4: &str = "agent4ykfcyj8fl6xzs88tsls05x93gmq68a7km05m4j";
pub const AGENT5: &str = "agent5k5k7y4hgy5lkq0kj3k3e9k38lquh0m66kxsu5c";

pub const AGENT_BENEFICIARY: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const ADMIN: &str = "cosmos1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u0tvx7u";
pub const ANYONE: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT0: &str = "cosmos1055rfv3fv0zxsp8h3x88mctnm7x9mlgmf4m4d6";
pub const PARTICIPANT1: &str = "cosmos1c3cy3wzzz3698ypklvh7shksvmefj69xhm89z2";
pub const PARTICIPANT2: &str = "cosmos1far5cqkvny7k9wq53aw0k42v3f76rcylzzv05n";
pub const PARTICIPANT3: &str = "cosmos1xj3xagnprtqpfnvyp7k393kmes73rpuxqgamd8";
pub const PARTICIPANT4: &str = "cosmos1t5u0jfg3ljsjrh2m9e47d4ny2hea7eehxrzdgd";
pub const PARTICIPANT5: &str = "cosmos1k5k7y4hgy5lkq0kj3k3e9k38lquh0m66kxsu5c";
pub const PARTICIPANT6: &str = "cosmos14a8clxc49z9e3mjzhamhkprt2hgf0y53zczzj0";
pub const VERY_RICH: &str = "cosmos1c3cy3wzzz3698ypklvh7shksvmefj69xhm89z2";
pub const NATIVE_DENOM: &str = "atom";
pub const TWO_MINUTES: u64 = 120_000_000_000;

pub(crate) fn mock_instantiate(deps: DepsMut<Empty>) -> Result<Response, ContractError> {
    let msg = InstantiateMsg {
        native_denom: Some(NATIVE_DENOM.to_string()),
        owner_addr: None,
        agent_nomination_duration: Some(DEFAULT_NOMINATION_DURATION),
    };
    let info = mock_info("sender", &coins(1000, "meow"));
    instantiate(deps, mock_env(), info.clone(), msg)
}
pub(crate) fn mock_config() -> Config {
    Config {
        paused: false,
        owner_addr: Addr::unchecked(ADMIN),
        native_denom: NATIVE_DENOM.to_string(),
        min_tasks_per_agent: DEFAULT_MIN_TASKS_PER_AGENT,
        agent_nomination_duration: DEFAULT_NOMINATION_DURATION,
    }
}
