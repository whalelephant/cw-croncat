use cosmwasm_std::{to_vec, Binary, Deps, Empty, QueryRequest, StdError, StdResult, WasmQuery};
use serde_cw_value::Value;

pub(crate) fn query_wasm_smart_raw(
    deps: Deps,
    contract_addr: impl Into<String>,
    msg: Binary,
) -> StdResult<Binary> {
    let contract_addr = contract_addr.into();
    let request: QueryRequest<Empty> = QueryRequest::Wasm(WasmQuery::Smart { contract_addr, msg });

    // Copied from `QuerierWrapper::query`
    // because serde_json_wasm fails to deserialize slice into `serde_cw_value::Value`
    let raw = to_vec(&request).map_err(|serialize_err| {
        StdError::generic_err(format!("Serializing QueryRequest: {}", serialize_err))
    })?;
    let bin = match deps.querier.raw_query(&raw) {
        cosmwasm_std::SystemResult::Ok(cosmwasm_std::ContractResult::Ok(value)) => value,
        cosmwasm_std::SystemResult::Ok(cosmwasm_std::ContractResult::Err(contract_err)) => {
            return Err(StdError::generic_err(format!(
                "Querier contract error: {}",
                contract_err
            )));
        }
        cosmwasm_std::SystemResult::Err(system_err) => {
            return Err(StdError::generic_err(format!(
                "Querier system error: {}",
                system_err
            )));
        }
    };
    Ok(bin)
}

pub(crate) fn bin_to_value(bin: &[u8]) -> StdResult<Value> {
    cosmwasm_std::from_slice(bin)
        .map_err(|e| StdError::parse_err(std::any::type_name::<serde_cw_value::Value>(), e))
}
