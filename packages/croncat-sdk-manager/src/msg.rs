use crate::types::{Config, GasPrice, UpdateConfig};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cw20::{Cw20Coin, Cw20CoinVerified};

#[cw_serde]
pub struct ManagerInstantiateMsg {
    /// The native denominator of current chain
    pub denom: String,
    /// Address of the factory contract
    pub croncat_factory_addr: String,
    /// Name of the key for raw querying Tasks address from the factory
    pub croncat_tasks_key: (String, [u8; 2]),
    /// Name of the key for raw querying Agents address from the factory
    pub croncat_agents_key: (String, [u8; 2]),
    /// Address of the contract owner, defaults to the sender
    pub owner_addr: Option<String>,
    /// Gas prices that expected to be used by the agent
    pub gas_price: Option<GasPrice>,
    /// The duration a prospective agent has to nominate themselves.
    /// When a task is created such that a new agent can join,
    /// The agent at the zeroth index of the pending agent queue has this time to nominate
    /// The agent at the first index has twice this time to nominate (which would remove the former agent from the pending queue)
    /// Value is in seconds
    pub agent_nomination_duration: Option<u16>,

    /// Contract's treasury.
    /// Fees from tasks will go to this address, if set or to the owner address otherwise
    pub treasury_addr: Option<String>,
}

#[cw_serde]
pub enum ManagerExecuteMsg {
    /// Updates the croncat Config.
    /// Note: it's shared across contracts
    // Boxing cause of large enum variant
    UpdateConfig(Box<UpdateConfig>),
    /// Move balances from the manager to the owner address, or treasury_addr if set
    OwnerWithdraw {},
    /// Execute current task in the queue or task with queries if task_hash given
    ProxyCall {
        task_hash: Option<String>,
    },
    /// Receive native coins to include them to the task
    RefillTaskBalance {
        task_hash: String,
    },
    RefillTaskCw20Balance {
        task_hash: String,
        cw20: Cw20Coin,
    },
    /// Receive cw20 coin
    Receive(cw20::Cw20ReceiveMsg),
    /// Withdraw temp coins for users
    UserWithdraw {
        // In case user somehow manages to have too many coins we don't want them to get locked funds
        limit: Option<u64>,
    },
    /// Kick inactive agents
    Tick {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ManagerQueryMsg {
    /// Gets current croncat config
    #[returns(Config)]
    Config {},
    /// Gets manager available balances
    #[returns(cosmwasm_std::Uint128)]
    TreasuryBalance {},
    /// Gets Cw20 balances of the given wallet address
    #[returns(Vec<Cw20CoinVerified>)]
    UsersBalances {
        wallet: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}

#[cw_serde]
pub enum ManagerReceiveMsg {
    RefillTempBalance {},
    RefillTaskBalance { task_hash: String },
}
