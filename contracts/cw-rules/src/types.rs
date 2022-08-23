use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// We can import dao but for simplicity we show what we support
pub mod dao {
    pub use voting::status::Status;

    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryDao {
        Proposal { proposal_id: u64 },
    }

    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
    pub struct ProposalResponse {
        /// The ID of the proposal being returned.
        pub id: u64,
        pub proposal: AnyChoiceProposal,
    }
    //
    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
    pub struct AnyChoiceProposal {
        pub status: Status,
        //Ignore rest
    }
}

pub mod generic_query {
    use super::*;
    use crate::helpers::ValueOrdering;
    use cosmwasm_std::Binary;
    use serde_json::Value;
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
    pub struct GenericQuery {
        pub msg: Binary,
        pub gets: Vec<ValueIndex>,

        pub ordering: ValueOrdering,
        pub value: Value,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
    #[serde(untagged)]
    pub enum ValueIndex {
        Key(String),
        Number(u64),
    }
}
