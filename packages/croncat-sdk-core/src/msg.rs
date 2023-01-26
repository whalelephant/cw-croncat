use crate::types::{BalancesResponse, Config, GasPrice, UpdateConfig};
use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Uint64};
use cw20::Cw20Coin;

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
    pub owner_id: Option<String>,
    /// Gas needed to cover ExecuteMsg::ProxyCall without any action
    pub gas_base_fee: Option<Uint64>,
    /// Gas needed to cover single non-wasm task's Action
    pub gas_action_fee: Option<Uint64>,
    /// Gas needed to cover single non-wasm query
    pub gas_query_fee: Option<Uint64>,
    /// Gas needed to cover single wasm query
    pub gas_wasm_query_fee: Option<Uint64>,
    /// Gas prices that expected to be used by the agent
    pub gas_price: Option<GasPrice>,
    /// The duration a prospective agent has to nominate themselves.
    /// When a task is created such that a new agent can join,
    /// The agent at the zeroth index of the pending agent queue has this time to nominate
    /// The agent at the first index has twice this time to nominate (which would remove the former agent from the pending queue)
    /// Value is in seconds
    pub agent_nomination_duration: Option<u16>,
}

#[cw_serde]
pub enum ManagerExecuteMsg {
    /// Updates the croncat Config.
    /// Note: it's shared across contracts
    UpdateConfig(UpdateConfig),
    /// Move balances from the manager to the address
    OwnerWithdraw {
        native_balances: Vec<Coin>,
        cw20_balances: Vec<Cw20Coin>,
    },
    /// Execute current task in the queue or task with queries if task_hash given
    ProxyCall { task_hash: Option<String> },
    /// Receive native coins to include them to the task
    RefillNativeBalance {},
    /// Receive cw20 coin
    Receive(cw20::Cw20ReceiveMsg),
    /// Withdraw temp coins for users
    UserWithdraw {
        native_balances: Vec<Coin>,
        cw20_balances: Vec<Cw20Coin>,
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
    #[returns(BalancesResponse)]
    AvailableBalances {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    /// Gets Cw20 balances of the given wallet address
    #[returns(BalancesResponse)]
    UsersBalances {
        wallet: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}

#[cw_serde]
pub enum ManagerReceiveMsg {
    RefillCw20Balance {},
}
