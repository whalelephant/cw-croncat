use crate::types::{GasPrice, UpdateConfig};
use cosmwasm_schema::{cw_serde, QueryResponses};
use croncat_sdk_core::hooks::{hook_messages::*, hooks::*};

use cw20::Cw20Coin;

#[cw_serde]
pub struct ManagerInstantiateMsg {
    /// The native denominator of current chain
    pub denom: String,
    /// CW2 Version provided by factory
    pub version: Option<String>,
    /// Name of the key for raw querying Tasks address from the factory
    pub croncat_tasks_key: (String, [u8; 2]),
    /// Name of the key for raw querying Agents address from the factory
    pub croncat_agents_key: (String, [u8; 2]),
    /// Address of the contract owner, defaults to the sender
    pub owner_addr: Option<String>,
    /// Gas prices that expected to be used by the agent
    pub gas_price: Option<GasPrice>,

    /// Contract's treasury.
    /// Fees from tasks will go to this address, if set or to the owner address otherwise
    pub treasury_addr: Option<String>,

    /// List of whitelisted cw20s
    pub cw20_whitelist: Option<Vec<String>>,
}

#[cw_serde]
pub enum ManagerExecuteMsg {
    /// Updates the croncat Config.
    /// Note: it's shared across contracts
    // Boxing cause of large enum variant
    UpdateConfig(Box<UpdateConfig>),

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

    /// Create task's balance, called by the tasks contract
    CreateTaskBalanceHook(CreateTaskBalanceHookMsg),

    /// Remove task's balance, called by the tasks contract
    RemoveTaskHook(RemoveTaskHookMsg),

    /// Move balances from the manager to the owner address, or treasury_addr if set
    OwnerWithdraw {},

    /// Withdraw temp coins for users
    UserWithdraw {
        // In case user somehow manages to have too many coins we don't want them to get locked funds
        limit: Option<u64>,
    },

    /// Withdraw agent rewards on agent removal, this should be called only by agent contract if passing WithdrawAgentRewardsHookMsg,
    /// or None for regular user call
    WithdrawAgentRewardsHook(Option<WithdrawAgentRewardsHookMsg>),

    /// Function for adding hooks
    AddHook { prefix: String, addr: String },
    /// Function for removing hooks
    RemoveHook { prefix: String, addr: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum ManagerQueryMsg {
    /// Gets current croncat config
    #[returns(crate::types::Config)]
    Config {},
    /// Gets manager available balances
    #[returns(cosmwasm_std::Uint128)]
    TreasuryBalance {},
    /// Gets Cw20 balances of the given wallet address
    #[returns(Vec<cw20::Cw20CoinVerified>)]
    UsersBalances {
        address: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    /// Get task balance
    #[returns(crate::types::TaskBalanceResponse)]
    TaskBalance { task_hash: String },

    #[returns(cosmwasm_std::Uint128)]
    AgentRewards { agent_id: String },

    #[returns(HooksResponse)]
    Hooks { prefix:String },
}

#[cw_serde]
pub enum ManagerReceiveMsg {
    RefillTempBalance {},
    RefillTaskBalance { task_hash: String },
}

