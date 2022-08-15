use cosmwasm_std::{Addr, Coin};
use cw20::Balance;
use cw_croncat_core::types::Rule;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    QueryResult {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Individual query evaluations
    GetBalance {
        address: Addr,
        denom: String,
    },
    GetCW20Balance {
        cw20_contract: Addr,
        address: Addr,
    },
    HasBalance {
        balance: Balance,
        required_balance: Balance,
    },
    CheckOwnerOfNFT {
        address: Addr,
        nft_address: Addr,
        token_id: String,
    },
    CheckProposalReadyToExec {
        dao_address: Addr,
        proposal_id: String,
    },
    // // Full evaluations
    // QueryConstruct {
    //     rules: Vec<Rule>,
    // },
}

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryMultiResponse {
    pub data: Vec<String>,
}

pub type RuleResponse<T> = (bool, T);
