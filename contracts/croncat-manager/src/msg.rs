use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Uint64;
use croncat_sdk_core::types::{BalancesResponse, Config, GasPrice, UpdateConfig};
use cw20::{Cw20Coin, Cw20CoinVerified};

#[cw_serde]
pub struct InstantiateMsg {
    pub denom: String,
    pub cw_rules_addr: String,
    pub croncat_tasks_addr: String,
    pub croncat_agents_addr: String,
    pub owner_id: Option<String>,
    pub gas_base_fee: Option<Uint64>,
    pub gas_action_fee: Option<Uint64>,
    pub gas_query_fee: Option<Uint64>,
    pub gas_wasm_query_fee: Option<Uint64>,
    pub gas_price: Option<GasPrice>,
    pub agent_nomination_duration: Option<u16>,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig(UpdateConfig),
    // TODO:
    // MoveBalances {
    //     balances: Vec<Balance>,
    //     account_id: String,
    // },
    ProxyCall {
        task_hash: Option<String>,
    },
    /// Receive cw20 token
    Receive(cw20::Cw20ReceiveMsg),
    WithdrawWalletBalances {
        cw20_amounts: Vec<Cw20Coin>,
    },
    Tick {},
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Config)]
    Config {},
    #[returns(BalancesResponse)]
    Balances {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    #[returns(Vec<Cw20CoinVerified>)]
    Cw20WalletBalances {
        wallet: String,
        from_index: Option<u64>,
        limit: Option<u64>,
    },
}
