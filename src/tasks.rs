// use crypto::digest::Digest;
// use crypto::sha3::Sha3;
use hex::encode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// use crate::traits::Hash;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Timestamp};
use cw20::Balance;

/// Defines the spacing of execution
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum Interval {
    /// For when this is a non-recurring future scheduled TXN
    Once,

    /// The ugly batch schedule type, in case you need to exceed single TXN gas limits, within fewest block(s)
    Immediate,

    /// Crontab Spec String
    Cron(String),

    /// Allows timing based on block intervals rather than timestamps
    Block(u64),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum BoundarySpec {
    /// Represents the block height
    Height(u64),

    /// Represents the block timestamp
    Time(Timestamp),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Boundary {
    ///
    pub start: Option<BoundarySpec>,
    ///
    pub end: Option<BoundarySpec>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Rule {
    /// TBD: Interchain query support (See ibc::IbcMsg)
    pub chain_id: Option<String>,

    /// Account to direct all view calls against
    pub contract_id: Addr,

    // NOTE: Only allow static pre-defined query msg
    pub msg: Binary,
}

/// The response required by all rule queries. Bool is needed for croncat, T allows flexible rule engine
pub type RuleResponse<T> = (bool, T);

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Task {
    /// Entity responsible for this task, can change task details
    pub owner_id: Addr,

    /// Scheduling definitions
    pub interval: Interval,
    pub boundary: Boundary,

    /// Defines if this task can continue until balance runs out
    pub stop_on_fail: bool,

    /// NOTE: Only tally native balance here, manager can maintain token/balances outside of tasks
    pub total_deposit: Balance,

    /// The cosmos message to call, if time or rules are met
    pub action: CosmosMsg,
    // TODO: Decide if batch should be supported? Does that break gas limits ESP when rules are applied?
    // pub action: Vec<CosmosMsg>,
    /// A prioritized list of messages that can be chained decision matrix
    /// required to complete before task action
    /// Rules MUST return the ResolverResponse type
    pub rules: Option<Vec<Rule>>,
}

impl Task {
    pub fn to_hash(&self) -> String {
        let message = format!(
            "{:?}{:?}{:?}{:?}{:?}",
            self.owner_id, self.interval, self.boundary, self.action, self.rules
        );

        let hash = Sha256::digest(message.as_bytes());
        encode(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, BankMsg, CosmosMsg};
    use cw20::Balance;

    #[test]
    fn task_to_hash_success() {
        let to_address = String::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send { to_address, amount };
        let msg: CosmosMsg = bank.clone().into();

        let task = Task {
            owner_id: Addr::unchecked("nobody".to_string()),
            interval: Interval::Immediate,
            boundary: Boundary {
                start: None,
                end: None,
            },
            stop_on_fail: false,
            total_deposit: Balance::default(),
            action: msg,
            rules: None,
        };

        // HASH IT!
        let hash = task.to_hash();
        assert_eq!(
            "2e87eb9d9dd92e5a903eacb23ce270676e80727bea1a38b40646be08026d05bc",
            hash
        );
    }
}
