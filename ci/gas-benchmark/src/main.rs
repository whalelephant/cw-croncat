mod helpers;
mod report;
mod test_cases;
mod test_cases_with_rules;
mod types;

use anyhow::Result;
use cosm_orc::{config::cfg::Config, orchestrator::cosm_orc::CosmOrc};
use types::Account;

use crate::{
    helpers::{
        complete_tasks_for_three_times, init_contracts, key_addr_from_account,
        query_tasks_with_rules, refill_cw20, register_agent,
    },
    report::{
        average_base_gas_for_proxy_call, average_gas_for_one_native_ujunox, cost_approxes,
        max_gas_non_wasm_action,
    },
    test_cases::{
        complete_simple_task, delegate_to_bob_and_alice_recurring, delegate_to_bob_recurring,
        delegate_to_validator, delegate_to_validator_twice, send_cw20_to_bob_and_alice_recurring,
        send_cw20_to_bob_recurring, send_cw20_to_insertable_addr, send_to_bob_and_alice_recurring,
        send_to_bob_recurring,
    },
    test_cases_with_rules::complete_simple_rule,
    types::ApproxGasCosts,
};

const RULES_NAME: &str = "cw_rules";
const CRONCAT_NAME: &str = "cw_croncat";
const CW20_NAME: &str = "cw20_base";
const BOB_ADDR: &str = "juno14vhcdsyf83ngsrrqc92kmw8q9xakqjm0ff2dpn";
const ALICE_ADDR: &str = "juno1l8hl8e0ut8jdaecxwazs9m32ak02ez4rssq4wl";

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
    let (user_key, user_addr) = key_addr_from_account(user_account);

    init_contracts(&mut orc, &admin_key, &admin_addr, &user_addr, &denom)?;
    register_agent(&mut orc, &agent_key)?;
    let cw20_addr = orc.contract_map.address(CW20_NAME)?;
    refill_cw20(&mut orc, &user_key, 100_000)?;
    // TEST IT WORKS
    let _ = complete_simple_task(&mut orc, (&agent_key, &agent_addr), &user_key, &denom)?;
    let _ = complete_simple_rule(&mut orc, (&agent_key, &agent_addr), &user_key, &denom)?;

    let tasks = vec![
        // Send tasks
        (send_to_bob_recurring(&denom), 100, "send_one_native"),
        (
            send_to_bob_and_alice_recurring(&denom),
            100,
            "send_two_native",
        ),
        // wasm(CW20 send) tasks
        (
            send_cw20_to_bob_recurring(&cw20_addr, 3),
            100,
            "send_single_cw20",
        ),
        (
            send_cw20_to_bob_and_alice_recurring(&cw20_addr, 3),
            100,
            "send_two_cw20",
        ),
        (delegate_to_validator(&denom), 100, "delegate_once"),
        (delegate_to_validator_twice(&denom), 100, "delegate_twice"),
        // Failed Stake tasks
        (
            delegate_to_bob_recurring(&denom),
            100,
            "failed_delegate_once",
        ),
        (
            delegate_to_bob_and_alice_recurring(&denom),
            100,
            "failed_delegate_twice",
        ),
    ];
    let gas_fees_usage = complete_tasks_for_three_times(
        &mut orc,
        (&agent_key, &agent_addr),
        &user_key,
        &denom,
        tasks,
    )?;
    let cost_per_send = cost_approxes(&gas_fees_usage[0], &gas_fees_usage[1]);
    println!("bank send reports:");
    println!("approx_base_gas: {}", cost_per_send.approx_base_gas());
    println!(
        "approx_gas_for_unregister: {}",
        cost_per_send.approx_gas_for_unregister()
    );
    println!(
        "approx_gas_per_action: {}\n",
        cost_per_send.approx_gas_per_action()
    );

    let cost_per_cw20 = cost_approxes(&gas_fees_usage[2], &gas_fees_usage[3]);
    println!("wasm reports:");
    println!("approx_base_gas: {}", cost_per_cw20.approx_base_gas());
    println!(
        "approx_gas_for_unregister: {}",
        cost_per_cw20.approx_gas_for_unregister()
    );
    println!(
        "approx_gas_per_action: {}\n",
        cost_per_cw20.approx_gas_per_action()
    );

    let cost_per_delegate = cost_approxes(&gas_fees_usage[4], &gas_fees_usage[5]);
    println!("delegate reports:");
    println!("approx_base_gas: {}", cost_per_delegate.approx_base_gas());
    println!(
        "approx_gas_for_unregister: {}",
        cost_per_delegate.approx_gas_for_unregister()
    );
    println!(
        "approx_gas_per_action: {}\n",
        cost_per_delegate.approx_gas_per_action()
    );

    let cost_per_failed_delegate = cost_approxes(&gas_fees_usage[6], &gas_fees_usage[7]);
    println!("failed delegate reports:");
    println!(
        "approx_base_gas: {}",
        cost_per_failed_delegate.approx_base_gas()
    );
    println!(
        "approx_gas_for_unregister: {}",
        cost_per_failed_delegate.approx_gas_for_unregister()
    );
    println!(
        "max_gas_per_action: {}",
        cost_per_failed_delegate.max_gas_per_action()
    );
    println!(
        "approx_gas_per_action: {}\n",
        cost_per_failed_delegate.approx_gas_per_action()
    );

    let non_wasm_reports: Vec<ApproxGasCosts> =
        vec![cost_per_send, cost_per_failed_delegate, cost_per_delegate];
    let wasm_reports: Vec<ApproxGasCosts> = vec![cost_per_cw20];
    let together_reports: Vec<ApproxGasCosts> = non_wasm_reports
        .iter()
        .cloned()
        .chain(wasm_reports.iter().cloned())
        .collect();
    println!(
        "max_gas_non_wasm_action: {}",
        max_gas_non_wasm_action(&non_wasm_reports)
    );
    println!(
        "average_base_gas_for_proxy_call: {}",
        average_base_gas_for_proxy_call(&together_reports)
    );
    let all_tasks_info = gas_fees_usage.into_iter().flatten().collect();
    println!(
        "avg_gas_cost: {}",
        average_gas_for_one_native_ujunox(all_tasks_info)
    );

    // Tests
    {
        let croncat_addr = orc.contract_map.address(CRONCAT_NAME)?;
        // create task with insertable addr transfer
        helpers::create_task(
            send_cw20_to_insertable_addr(&croncat_addr, &cw20_addr, &admin_addr),
            &mut orc,
            &user_key,
            "send_to_insertable_addr",
            &denom,
            500_000,
        )
        .unwrap();
        let tasks = query_tasks_with_rules(&orc)?;
        let task_hash = tasks[0].task_hash.clone();

        // Change the owner
        let new_owner_id = accounts[3].clone();
        orc.execute(
            CRONCAT_NAME,
            "update_config",
            &cw_croncat::ExecuteMsg::UpdateSettings {
                owner_id: Some(new_owner_id.address.clone()),
                slot_granularity_time: None,
                paused: None,
                agent_fee: None,
                gas_base_fee: None,
                gas_action_fee: None,
                gas_fraction: None,
                proxy_callback_gas: None,
                min_tasks_per_agent: None,
                agents_eject_threshold: None,
            },
            &admin_key,
            vec![],
        )
        .unwrap();

        orc.poll_for_n_blocks(1, std::time::Duration::from_millis(20_000), false)
            .unwrap();

        helpers::proxy_call_with_hash(
            &mut orc,
            &agent_key,
            task_hash,
            "proxy_call_tak_with_intertable_addr",
        )
        .unwrap();

        let balance_res: cw20::BalanceResponse = orc
            .query(
                CW20_NAME,
                &cw20_base::msg::QueryMsg::Balance {
                    address: new_owner_id.address,
                },
            )?
            .data()?;
        if balance_res.balance != cosmwasm_std::Uint128::new(10) {
            return Err(anyhow::anyhow!("Not transfered"));
        }
    }
    let gas_report_dir = std::env::var("GAS_OUT_DIR").unwrap_or_else(|_| "gas_reports".to_string());
    save_gas_report(&orc, &gas_report_dir);

    Ok(())
}

fn save_gas_report(orc: &CosmOrc, gas_report_dir: &str) {
    let report = orc
        .gas_profiler_report()
        .expect("error fetching profile reports");

    let s = serde_json::to_string(report).unwrap();

    let p = std::path::Path::new(gas_report_dir);
    if !p.exists() {
        std::fs::create_dir(p).unwrap();
    }

    let file_name = "gas_report.json";
    std::fs::write(p.join(file_name), s).unwrap();
}
