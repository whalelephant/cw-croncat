use cw_croncat_core::{
    msg::QueryConstruct,
    types::{CheckOwnerOfNFT, CheckProposalStatus, HasBalanceGte},
};
use generic_query::GenericQuery;
//use cw_croncat_core::types::Rule;
//use cosmwasm_std::Coin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    QueryResult {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Individual query evaluations
    GetBalance {
        address: String,
        denom: String,
    },
    GetCW20Balance {
        cw20_contract: String,
        address: String,
    },
    HasBalanceGte(HasBalanceGte),
    CheckOwnerOfNFT(CheckOwnerOfNFT),
    CheckProposalStatus(CheckProposalStatus),
    GenericQuery(GenericQuery),
    // Full evaluations
    QueryConstruct(QueryConstruct),
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct QueryMultiResponse {
    pub data: Vec<String>,
}

pub type RuleResponse<T> = (bool, T);
