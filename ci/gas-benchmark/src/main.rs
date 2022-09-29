mod helpers;
mod test_cases;
mod types;

use anyhow::Result;
use cosm_orc::{config::cfg::Config, orchestrator::cosm_orc::CosmOrc};
use types::Account;

use crate::{
    helpers::{init_contracts, key_addr_from_account, register_agent},
    test_cases::{complete_reccuring_task, complete_simple_task},
};

const RULES_NAME: &str = "cw_rules";
const CRONCAT_NAME: &str = "cw_croncat";
const BOB_ADDR: &str = "juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn";

fn main() -> Result<()> {
    env_logger::init();

    let cfg = Config::from_yaml("ci/local_config.yaml")?;
    let denom = cfg.chain_cfg.denom.clone();
    let mut orc = CosmOrc::new(cfg, true)?;
    let accounts: Vec<Account> = serde_json::from_slice(&std::fs::read("ci/test_accounts.json")?)?;
    let admin_account = accounts[0].clone();
    let agent_account = accounts[1].clone();
    let user_account = accounts[2].clone();

    let (admin_key, admin_addr) = key_addr_from_account(admin_account);
    let (agent_key, agent_addr) = key_addr_from_account(agent_account);
    let (user_key, _user_addr) = key_addr_from_account(user_account);

    init_contracts(&mut orc, &admin_key, &admin_addr, &denom)?;
    register_agent(&mut orc, &agent_key)?;
    let simple_gas = complete_simple_task(&mut orc, (&agent_key, &agent_addr), &user_key, &denom)?;
    let recurring_gas =
        complete_reccuring_task(&mut orc, (&agent_key, &agent_addr), &user_key, &denom)?;
    println!("simple: {simple_gas:?}");
    println!("recurring_gas: {recurring_gas:?}");
    Ok(())
}
