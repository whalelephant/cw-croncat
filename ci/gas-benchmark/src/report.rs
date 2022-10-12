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
    }
}
