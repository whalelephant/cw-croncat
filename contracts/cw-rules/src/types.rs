use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// We can import dao but for simplicity we show what we support
pub mod dao {
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

    #[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Copy)]
    #[serde(rename_all = "snake_case")]
    #[repr(u8)]
    pub enum Status {
        /// The proposal is open for voting.
        Open,
        /// The proposal has been rejected.
        Rejected,
        /// The proposal has been passed but has not been executed.
        Passed,
        /// The proposal has been passed and executed.
        Executed,
        /// The proposal has failed or expired and has been closed. A
        /// proposal deposit refund has been issued if applicable.
        Closed,
        // The proposal has failed during execution
        ExecutionFailed,
    }
}
