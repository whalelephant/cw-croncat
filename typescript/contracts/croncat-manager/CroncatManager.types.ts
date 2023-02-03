/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/

export interface InstantiateMsg {
  croncat_agents_key: [string, [number, number]];
  croncat_tasks_key: [string, [number, number]];
  cw20_whitelist?: string[] | null;
  denom: string;
  gas_price?: GasPrice | null;
  owner_addr?: string | null;
  treasury_addr?: string | null;
  version?: string | null;
}
export interface GasPrice {
  denominator: number;
  gas_adjustment_numerator: number;
  numerator: number;
}
export type ExecuteMsg = {
  update_config: UpdateConfig;
} | {
  owner_withdraw: {};
} | {
  proxy_call: {
    task_hash?: string | null;
  };
} | {
  refill_task_balance: {
    task_hash: string;
  };
} | {
  refill_task_cw20_balance: {
    cw20: Cw20Coin;
    task_hash: string;
  };
} | {
  receive: Cw20ReceiveMsg;
} | {
  user_withdraw: {
    limit?: number | null;
  };
} | {
  create_task_balance: ManagerCreateTaskBalance;
} | {
  remove_task: ManagerRemoveTask;
} | {
  withdraw_agent_rewards: WithdrawRewardsOnRemovalArgs | null;
};
export type Uint128 = string;
export type Binary = string;
export type Addr = string;
export interface UpdateConfig {
  agent_fee?: number | null;
  croncat_agents_key?: [string, [number, number]] | null;
  croncat_tasks_key?: [string, [number, number]] | null;
  cw20_whitelist?: string[] | null;
  gas_price?: GasPrice | null;
  owner_addr?: string | null;
  paused?: boolean | null;
  treasury_addr?: string | null;
  treasury_fee?: number | null;
}
export interface Cw20Coin {
  address: string;
  amount: Uint128;
}
export interface Cw20ReceiveMsg {
  amount: Uint128;
  msg: Binary;
  sender: string;
}
export interface ManagerCreateTaskBalance {
  amount_for_one_task: AmountForOneTask;
  cw20?: Cw20CoinVerified | null;
  recurring: boolean;
  sender: Addr;
  task_hash: number[];
}
export interface AmountForOneTask {
  coin: [Coin | null, Coin | null];
  cw20?: Cw20CoinVerified | null;
  gas: number;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface Cw20CoinVerified {
  address: Addr;
  amount: Uint128;
}
export interface ManagerRemoveTask {
  sender: Addr;
  task_hash: number[];
}
export interface WithdrawRewardsOnRemovalArgs {
  agent_id: string;
  payable_account_id: string;
}
export type QueryMsg = {
  config: {};
} | {
  treasury_balance: {};
} | {
  users_balances: {
    from_index?: number | null;
    limit?: number | null;
    wallet: string;
  };
} | {
  task_balance: {
    task_hash: string;
  };
} | {
  agent_rewards: {
    agent_id: string;
  };
};
export interface Config {
  agent_fee: number;
  croncat_agents_key: [string, [number, number]];
  croncat_factory_addr: Addr;
  croncat_tasks_key: [string, [number, number]];
  cw20_whitelist: Addr[];
  gas_price: GasPrice;
  limit: number;
  native_denom: string;
  owner_addr: Addr;
  paused: boolean;
  treasury_addr?: Addr | null;
  treasury_fee: number;
}
export interface TaskBalanceResponse {
  balance?: TaskBalance | null;
}
export interface TaskBalance {
  cw20_balance?: Cw20CoinVerified | null;
  ibc_balance?: Coin | null;
  native_balance: Uint128;
}
export type ArrayOfCw20CoinVerified = Cw20CoinVerified[];