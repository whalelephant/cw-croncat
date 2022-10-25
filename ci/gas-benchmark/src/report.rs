use crate::{
    helpers::average_u64_slice,
    types::{ApproxGasCosts, GasInformation},
};

pub(crate) fn cost_approxes(
    one_action: &[GasInformation],
    two_actions: &[GasInformation],
) -> ApproxGasCosts {
    let mut diffs = vec![];
    for i in 0..one_action.len() {
        let diff = two_actions[i].gas_used - one_action[i].gas_used;
        diffs.push(diff);
    }
    let gas_per_action = average_u64_slice(&diffs);
    let max_gas_per_action = diffs.iter().max().unwrap().to_owned();
    let mut diffs = vec![];
    for item in one_action.iter().take(one_action.len() - 1) {
        let diff = item.gas_used - gas_per_action;
        diffs.push(diff);
    }
    let gas_for_proxy_call = average_u64_slice(&diffs);
    let diffs = [
        one_action
            .last()
            .unwrap()
            .gas_used
            .saturating_sub(gas_for_proxy_call + gas_per_action),
        two_actions
            .last()
            .unwrap()
            .gas_used
            .saturating_sub(gas_for_proxy_call + gas_per_action * 2),
    ];
    let gas_for_task_unregister = average_u64_slice(&diffs);
    ApproxGasCosts {
        gas_per_action,
        gas_for_proxy_call,
        gas_for_task_unregister,
        max_gas_per_action,
    }
}

pub(crate) fn average_gas_for_one_native_ujunox(gas_fees_usage: Vec<GasInformation>) -> u64 {
    let (total_gas, total_ujunox) = gas_fees_usage
        .into_iter()
        .fold((0, 0), |(gas, ujunox), info| {
            (gas + info.gas_used, ujunox + info.native_balance_burned)
        });
    (total_gas as f64 / total_ujunox as f64).ceil() as u64
}

pub(crate) fn max_gas_non_wasm_action(reports: &[ApproxGasCosts]) -> u64 {
    reports
        .iter()
        .fold(0, |cur, r| r.max_gas_per_action.max(cur))
}

pub(crate) fn average_base_gas_for_proxy_call(reports: &[ApproxGasCosts]) -> u64 {
    let arr: Vec<u64> = reports
        .iter()
        .map(|r| r.gas_for_proxy_call + r.gas_for_task_unregister)
        .collect();
    average_u64_slice(&arr)
}
