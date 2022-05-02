use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    // TODO: Submit issue for AppBuilder tests not working for -- deps.querier.query_bonded_denom()?;
    pub denom: String,
    pub owner_id: Option<Addr>,
}

// TODO: Check this !!!!!!!!!
// // Exporting a nice schema
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub enum Croncat {
//     Agent(Agent),
//     Config(Config),
//     Task(Task),
//     ConfigResponse(ConfigResponse),
//     BalancesResponse(BalancesResponse),
// }
