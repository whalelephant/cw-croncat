use crate::error::CoreError;
use crate::traits::Intervals;
use crate::types::{
    Action, AgentStatus, BalancerMode, Boundary, CheckedBoundary, GasPrice, GenericBalance,
    Interval, Task, Transform,
};
use crate::types::{Agent, SlotType};
use cosmwasm_std::{Addr, Coin, Timestamp, Uint64};
use cw20::{Balance, Cw20Coin, Cw20CoinVerified};
use cw_rules_core::types::CroncatQuery;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// NOTE: Which version is more practical?
// // Exporting a nice schema
// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// pub enum Croncat {
//     Agent(Agent),
//     Task(Task),
//     ConfigResponse(GetConfigResponse),
//     BalancesResponse(GetBalancesResponse),
//     GetAgentIdsResponse(GetAgentIdsResponse),
//     GetAgentTasksResponse(GetAgentTasksResponse),
//     TaskRequest(TaskRequest),
//     TaskResponse(TaskResponse),
//     ValidateIntervalResponse(ValidateIntervalResponse),
//     GetAgentResponse(GetAgentResponse),
//     GetTasksResponse(GetTasksResponse),
//     GetTasksByOwnerResponse(GetTasksByOwnerResponse),
//     GetTaskResponse(GetTaskResponse),
//     GetTaskHashResponse(GetTaskHashResponse),
//     GetSlotHashesResponse(GetSlotHashesResponse),
//     GetSlotIdsResponse(GetSlotIdsResponse),
// }

// Exporting a nice schema
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Croncat {
    pub(crate) agent: Option<Agent>,
    pub(crate) task: Option<Task>,
    pub(crate) config_response: Option<GetConfigResponse>,
    pub(crate) balance_response: Option<GetBalancesResponse>,
    pub(crate) get_agent_ids_response: Option<GetAgentIdsResponse>,
    pub(crate) get_agent_tasks_response: Option<AgentTaskResponse>,
    pub(crate) task_request: Option<TaskRequest>,
    pub(crate) task_response: Option<TaskResponse>,
    pub(crate) validate_interval_response: Option<bool>,
    pub(crate) get_agent_response: Option<Option<AgentResponse>>,
    pub(crate) get_tasks_response: Option<Vec<TaskResponse>>,
    pub(crate) get_tasks_by_owner_response: Option<Vec<TaskResponse>>,
    pub(crate) get_task_response: Option<Option<TaskResponse>>,
    pub(crate) get_task_hash_response: Option<String>,
    pub(crate) get_slot_hashes_response: Option<GetSlotHashesResponse>,
    pub(crate) get_slot_ids_response: Option<GetSlotIdsResponse>,
    pub(crate) get_wallet_balances_response: Option<GetWalletBalancesResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    // TODO: Submit issue for AppBuilder tests not working for -- deps.querier.query_bonded_denom()?;
    pub denom: String,
    pub cw_rules_addr: String,
    pub owner_id: Option<String>,
    pub gas_base_fee: Option<Uint64>,
    pub gas_action_fee: Option<Uint64>,
    pub gas_query_fee: Option<Uint64>,
    pub gas_wasm_query_fee: Option<Uint64>,
    pub gas_price: Option<GasPrice>,
    pub agent_nomination_duration: Option<u16>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateSettings {
        owner_id: Option<String>,
        slot_granularity_time: Option<u64>,
        paused: Option<bool>,
        agent_fee: Option<u64>,
        gas_base_fee: Option<Uint64>,
        gas_action_fee: Option<Uint64>,
        gas_query_fee: Option<Uint64>,
        gas_wasm_query_fee: Option<Uint64>,
        gas_price: Option<GasPrice>,
        proxy_callback_gas: Option<u32>,
        min_tasks_per_agent: Option<u64>,
        agents_eject_threshold: Option<u64>,
        // treasury_id: Option<String>,
    },
    MoveBalances {
        balances: Vec<Balance>,
        account_id: String,
    },

    RegisterAgent {
        payable_account_id: Option<String>,
    },
    UpdateAgent {
        payable_account_id: String,
    },
    CheckInAgent {},
    UnregisterAgent {
        from_behind: Option<bool>,
    },
    WithdrawReward {},

    CreateTask {
        task: TaskRequest,
    },
    RemoveTask {
        task_hash: String,
    },
    RefillTaskBalance {
        task_hash: String,
    },
    RefillTaskCw20Balance {
        task_hash: String,
        cw20_coins: Vec<Cw20Coin>,
    },
    ProxyCall {
        task_hash: Option<String>,
    },
    /// Receive cw20 token
    Receive(cw20::Cw20ReceiveMsg),
    WithdrawWalletBalance {
        cw20_amounts: Vec<Cw20Coin>,
    },
    Tick {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetConfig {},
    GetBalances {},
    GetAgent {
        account_id: String,
    },
    GetAgentIds {},
    GetAgentTasks {
        account_id: String,
    },
    GetTasks {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    GetTasksWithQueries {
        from_index: Option<u64>,
        limit: Option<u64>,
    },
    GetTasksByOwner {
        owner_id: String,
    },
    GetTask {
        task_hash: String,
    },
    GetTaskHash {
        task: Box<Task>,
    },
    ValidateInterval {
        interval: Interval,
    },
    GetSlotHashes {
        slot: Option<u64>,
    },
    GetSlotIds {},
    GetWalletBalances {
        wallet: String,
    },
    SimulateTask {
        task: TaskRequest,
        funds: Option<Vec<Coin>>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetConfigResponse {
    pub paused: bool,
    pub owner_id: Addr,
    // pub treasury_id: Option<Addr>,
    pub min_tasks_per_agent: u64,
    pub agents_eject_threshold: u64,
    pub agent_active_indices: Vec<(SlotType, u32, u32)>,
    pub agent_nomination_duration: u16,

    pub cw_rules_addr: Addr,

    pub agent_fee: u64,
    pub gas_price: GasPrice,
    pub gas_base_fee: u64,
    pub gas_action_fee: u64,
    pub proxy_callback_gas: u32,
    pub slot_granularity_time: u64,

    pub cw20_whitelist: Vec<Addr>,
    pub native_denom: String,
    pub available_balance: GenericBalance, // tasks + rewards balances
    pub staked_balance: GenericBalance, // surplus that is temporary staking (to be used in conjunction with external treasury)

    // The default amount of tasks to query
    pub limit: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetBalancesResponse {
    pub native_denom: String,
    pub available_balance: GenericBalance,
    pub staked_balance: GenericBalance,
    pub cw20_whitelist: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct GetWalletBalancesResponse {
    pub cw20_balances: Vec<Cw20CoinVerified>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GetAgentIdsResponse {
    pub active: Vec<Addr>,
    pub pending: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AgentResponse {
    // This field doesn't exist in the Agent struct and is the only one that differs
    pub status: AgentStatus,
    pub payable_account_id: Addr,
    pub balance: GenericBalance,
    pub total_tasks_executed: u64,
    pub last_executed_slot: u64,
    pub register_start: Timestamp,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct AgentTaskResponse {
    pub num_block_tasks: Uint64,
    pub num_block_tasks_extra: Uint64,
    pub num_cron_tasks: Uint64,
    pub num_cron_tasks_extra: Uint64,
}

impl AgentTaskResponse {
    pub fn has_any_slot_tasks(&self, slot_kind: SlotType) -> bool {
        if self.num_of_slot_tasks(slot_kind) < 1u64 {
            return false;
        }
        true
    }
    pub fn num_of_slot_tasks(&self, slot_kind: SlotType) -> u64 {
        if slot_kind == SlotType::Block {
            return self.num_block_tasks.u64();
        }

        self.num_cron_tasks.u64()
    }
    pub fn has_any_slot_extra_tasks(&self, slot_kind: SlotType) -> bool {
        if self.num_of_slot_extra_tasks(slot_kind) < 1u64 {
            return false;
        }
        true
    }
    pub fn num_of_slot_extra_tasks(&self, slot_kind: SlotType) -> u64 {
        if slot_kind == SlotType::Block {
            return self.num_block_tasks_extra.u64();
        }

        self.num_cron_tasks_extra.u64()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskRequest {
    pub interval: Interval,
    pub boundary: Option<Boundary>,
    pub stop_on_fail: bool,
    pub actions: Vec<Action>,
    pub queries: Option<Vec<CroncatQuery>>,
    pub transforms: Option<Vec<Transform>>,
    pub cw20_coins: Vec<Cw20Coin>,
    pub sender: Option<String>,
}
pub struct TaskRequestBuilder {
    interval: Interval,
    boundary: Option<Boundary>,
    stop_on_fail: bool,
    actions: Option<Vec<Action>>,
    queries: Option<Option<Vec<CroncatQuery>>>,
    transforms: Option<Option<Vec<Transform>>>,
    cw20_coins: Option<Vec<Cw20Coin>>,
    sender: Option<String>,
}
#[allow(dead_code)]
impl TaskRequestBuilder {
    pub fn new() -> Self {
        Self {
            interval: Interval::Once,
            boundary: None,
            stop_on_fail: false,
            actions: None,
            queries: None,
            transforms: None,
            cw20_coins: None,
            sender: None,
        }
    }
    pub fn with_interval(&mut self, interval: Interval) -> &mut Self {
        self.interval = interval;
        self
    }
    pub fn once(&mut self) -> &mut Self {
        self.with_interval(Interval::Once)
    }
    pub fn block(&mut self, block_inerval: u64) -> &mut Self {
        self.with_interval(Interval::Block(block_inerval))
    }
    pub fn cron(&mut self, crontab: String) -> &mut Self {
        self.with_interval(Interval::Cron(crontab))
    }
    pub fn immediate(&mut self) -> &mut Self {
        self.with_interval(Interval::Immediate)
    }
    pub fn with_boundary(&mut self, boundary: Boundary) -> &mut Self {
        self.boundary = Some(boundary);
        self
    }
    pub fn with_time_boundary(&mut self, start: Timestamp, end: Timestamp) -> &mut Self {
        self.with_boundary(Boundary::Time {
            start: Some(start),
            end: Some(end),
        })
    }
    pub fn with_height_boundary(&mut self, start: u64, end: u64) -> &mut Self {
        self.with_boundary(Boundary::Height {
            start: Some(Uint64::new(start)),
            end: Some(Uint64::new(end)),
        })
    }
    pub fn should_stop_on_fail(&mut self, stop_on_fail: bool) -> &mut Self {
        self.stop_on_fail = stop_on_fail;
        self
    }

    pub fn with_action(&mut self, action: Action) -> &mut Self {
        self.actions = Some(vec![action]);
        self
    }
    pub fn with_actions(&mut self, actions: Vec<Action>) -> &mut Self {
        self.actions = Some(actions);
        self
    }
    pub fn with_query(&mut self, query: CroncatQuery) -> &mut Self {
        self.queries = Some(Some(vec![query]));
        self
    }
    pub fn with_queries(&mut self, queries: Vec<CroncatQuery>) -> &mut Self {
        self.queries = Some(Some(queries));
        self
    }
    pub fn with_transform(&mut self, transform: Transform) -> &mut Self {
        self.transforms = Some(Some(vec![transform]));
        self
    }
    fn with_transforms(&mut self, transforms: Vec<Transform>) -> &mut Self {
        self.transforms = Some(Some(transforms));
        self
    }
    fn with_cw20(&mut self, cw20: Cw20Coin) -> &mut Self {
        self.cw20_coins = Some(vec![cw20]);
        self
    }
    fn with_sender(&mut self, sender: String) -> &mut Self {
        self.sender = Some(sender);
        self
    }
    pub fn with_cw20s(&mut self, cw20s: Vec<Cw20Coin>) -> &mut Self {
        self.cw20_coins = Some(cw20s);
        self
    }
    pub fn build(&self) -> Result<TaskRequest, CoreError> {
        if !self.interval.is_valid() {
            return Err(CoreError::InvalidInterval {});
        }
        CheckedBoundary::new(self.boundary, &self.interval)?;

        Ok(TaskRequest {
            interval: self.interval.clone(),
            boundary: self.boundary,
            stop_on_fail: self.stop_on_fail,
            actions: self.actions.clone().unwrap_or_default(),
            queries: self.queries.clone().unwrap_or_default(),
            transforms: self.transforms.clone().unwrap_or_default(),
            cw20_coins: self.cw20_coins.clone().unwrap_or_default(),
            sender: self.sender.clone(),
        })
    }
}
impl Default for TaskRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskResponse {
    pub task_hash: String,

    pub owner_id: Addr,

    pub interval: Interval,
    pub boundary: Option<Boundary>,

    pub stop_on_fail: bool,
    pub total_deposit: Vec<Coin>,
    pub total_cw20_deposit: Vec<Cw20CoinVerified>,
    pub amount_for_one_task_native: Vec<Coin>,
    pub amount_for_one_task_cw20: Vec<Cw20CoinVerified>,

    pub actions: Vec<Action>,
    pub queries: Option<Vec<CroncatQuery>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TaskWithQueriesResponse {
    pub task_hash: String,
    pub interval: Interval,
    pub boundary: Option<Boundary>,
    pub queries: Option<Vec<CroncatQuery>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CwCroncatResponse {
    pub agent_active_queue: Vec<Addr>,
    pub agent_pending_queue: Vec<Addr>,
    pub tasks: Vec<TaskResponse>,
    pub task_total: Uint64,
    pub reply_index: Uint64,

    pub agent_nomination_begin_time: Option<Timestamp>,

    pub balancer_mode: BalancerMode,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        let boundary = match (task.boundary, &task.interval) {
            (
                CheckedBoundary {
                    start: None,
                    end: None,
                    is_block_boundary: None,
                },
                _,
            ) => None,
            (
                CheckedBoundary {
                    start,
                    end,
                    is_block_boundary: _,
                },
                Interval::Cron(_),
            ) => Some(Boundary::Time {
                start: start.map(Timestamp::from_nanos),
                end: end.map(Timestamp::from_nanos),
            }),
            (
                CheckedBoundary {
                    start,
                    end,
                    is_block_boundary: _,
                },
                _,
            ) => Some(Boundary::Height {
                start: start.map(Into::into),
                end: end.map(Into::into),
            }),
        };
        TaskResponse {
            task_hash: task.to_hash(),
            owner_id: task.owner_id,
            interval: task.interval,
            boundary,
            stop_on_fail: task.stop_on_fail,
            total_deposit: task.total_deposit.native,
            total_cw20_deposit: task.total_deposit.cw20,
            amount_for_one_task_native: task.amount_for_one_task.native,
            amount_for_one_task_cw20: task.amount_for_one_task.cw20,
            actions: task.actions,
            queries: task.queries,
        }
    }
}

impl From<Task> for TaskWithQueriesResponse {
    fn from(task: Task) -> Self {
        let boundary = match (task.boundary, &task.interval) {
            (
                CheckedBoundary {
                    start: None,
                    end: None,
                    is_block_boundary: None,
                },
                _,
            ) => None,
            (
                CheckedBoundary {
                    start,
                    end,
                    is_block_boundary: _,
                },
                Interval::Cron(_),
            ) => Some(Boundary::Time {
                start: start.map(Timestamp::from_nanos),
                end: end.map(Timestamp::from_nanos),
            }),
            (
                CheckedBoundary {
                    start,
                    end,
                    is_block_boundary: _,
                },
                _,
            ) => Some(Boundary::Height {
                start: start.map(Into::into),
                end: end.map(Into::into),
            }),
        };
        TaskWithQueriesResponse {
            task_hash: task.to_hash(),
            interval: task.interval,
            boundary,
            queries: task.queries,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct GetSlotHashesResponse {
    pub block_id: u64,
    pub block_task_hash: Vec<String>,
    pub time_id: u64,
    pub time_task_hash: Vec<String>,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct SimulateTaskResponse {
    pub estimated_gas: u64,
    pub occurrences: u64,
    pub task_hash: String,
}
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
pub struct GetSlotIdsResponse {
    pub time_ids: Vec<u64>,
    pub block_ids: Vec<u64>,
}

// cw_rules
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryConstruct {
    pub queries: Vec<CroncatQuery>,
}
