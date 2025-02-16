"use strict";
/**
* This file was automatically generated by @cosmwasm/ts-codegen@0.19.0.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @cosmwasm/ts-codegen generate command to regenerate this file.
*/
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.CwCroncatCoreClient = exports.CwCroncatCoreQueryClient = void 0;
class CwCroncatCoreQueryClient {
    constructor(client, contractAddress) {
        this.getConfig = () => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_config: {}
            });
        });
        this.getBalances = () => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_balances: {}
            });
        });
        this.getAgent = ({ accountId }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_agent: {
                    account_id: accountId
                }
            });
        });
        this.getAgentIds = () => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_agent_ids: {}
            });
        });
        this.getAgentTasks = ({ accountId }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_agent_tasks: {
                    account_id: accountId
                }
            });
        });
        this.getTasks = ({ fromIndex, limit }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_tasks: {
                    from_index: fromIndex,
                    limit
                }
            });
        });
        this.getTasksWithQueries = ({ fromIndex, limit }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_tasks_with_queries: {
                    from_index: fromIndex,
                    limit
                }
            });
        });
        this.getTasksByOwner = ({ ownerId }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_tasks_by_owner: {
                    owner_id: ownerId
                }
            });
        });
        this.getTask = ({ taskHash }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_task: {
                    task_hash: taskHash
                }
            });
        });
        this.getTaskHash = ({ task }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_task_hash: {
                    task
                }
            });
        });
        this.validateInterval = ({ interval }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                validate_interval: {
                    interval
                }
            });
        });
        this.getSlotHashes = ({ slot }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_slot_hashes: {
                    slot
                }
            });
        });
        this.getSlotIds = () => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_slot_ids: {}
            });
        });
        this.getWalletBalances = ({ wallet }) => __awaiter(this, void 0, void 0, function* () {
            return this.client.queryContractSmart(this.contractAddress, {
                get_wallet_balances: {
                    wallet
                }
            });
        });
        this.client = client;
        this.contractAddress = contractAddress;
        this.getConfig = this.getConfig.bind(this);
        this.getBalances = this.getBalances.bind(this);
        this.getAgent = this.getAgent.bind(this);
        this.getAgentIds = this.getAgentIds.bind(this);
        this.getAgentTasks = this.getAgentTasks.bind(this);
        this.getTasks = this.getTasks.bind(this);
        this.getTasksWithQueries = this.getTasksWithQueries.bind(this);
        this.getTasksByOwner = this.getTasksByOwner.bind(this);
        this.getTask = this.getTask.bind(this);
        this.getTaskHash = this.getTaskHash.bind(this);
        this.validateInterval = this.validateInterval.bind(this);
        this.getSlotHashes = this.getSlotHashes.bind(this);
        this.getSlotIds = this.getSlotIds.bind(this);
        this.getWalletBalances = this.getWalletBalances.bind(this);
    }
}
exports.CwCroncatCoreQueryClient = CwCroncatCoreQueryClient;
class CwCroncatCoreClient extends CwCroncatCoreQueryClient {
    constructor(client, sender, contractAddress) {
        super(client, contractAddress);
        this.updateSettings = ({ agentFee, agentsEjectThreshold, chainName, gasActionFee, gasBaseFee, gasPrice, gasQueryFee, gasWasmQueryFee, minTasksPerAgent, ownerId, paused, proxyCallbackGas, slotGranularityTime }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                update_settings: {
                    agent_fee: agentFee,
                    agents_eject_threshold: agentsEjectThreshold,
                    chain_name: chainName,
                    gas_action_fee: gasActionFee,
                    gas_base_fee: gasBaseFee,
                    gas_price: gasPrice,
                    gas_query_fee: gasQueryFee,
                    gas_wasm_query_fee: gasWasmQueryFee,
                    min_tasks_per_agent: minTasksPerAgent,
                    owner_id: ownerId,
                    paused,
                    proxy_callback_gas: proxyCallbackGas,
                    slot_granularity_time: slotGranularityTime
                }
            }, fee, memo, funds);
        });
        this.moveBalances = ({ accountId, balances }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                move_balances: {
                    account_id: accountId,
                    balances
                }
            }, fee, memo, funds);
        });
        this.registerAgent = ({ payableAccountId }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                register_agent: {
                    payable_account_id: payableAccountId
                }
            }, fee, memo, funds);
        });
        this.updateAgent = ({ payableAccountId }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                update_agent: {
                    payable_account_id: payableAccountId
                }
            }, fee, memo, funds);
        });
        this.checkInAgent = (fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                check_in_agent: {}
            }, fee, memo, funds);
        });
        this.unregisterAgent = ({ fromBehind }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                unregister_agent: {
                    from_behind: fromBehind
                }
            }, fee, memo, funds);
        });
        this.withdrawReward = (fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                withdraw_reward: {}
            }, fee, memo, funds);
        });
        this.createTask = ({ task }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                create_task: {
                    task
                }
            }, fee, memo, funds);
        });
        this.removeTask = ({ taskHash }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                remove_task: {
                    task_hash: taskHash
                }
            }, fee, memo, funds);
        });
        this.refillTaskBalance = ({ taskHash }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                refill_task_balance: {
                    task_hash: taskHash
                }
            }, fee, memo, funds);
        });
        this.refillTaskCw20Balance = ({ cw20Coins, taskHash }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                refill_task_cw20_balance: {
                    cw20_coins: cw20Coins,
                    task_hash: taskHash
                }
            }, fee, memo, funds);
        });
        this.proxyCall = ({ taskHash }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                proxy_call: {
                    task_hash: taskHash
                }
            }, fee, memo, funds);
        });
        this.receive = ({ amount, msg, sender }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                receive: {
                    amount,
                    msg,
                    sender
                }
            }, fee, memo, funds);
        });
        this.withdrawWalletBalance = ({ cw20Amounts }, fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                withdraw_wallet_balance: {
                    cw20_amounts: cw20Amounts
                }
            }, fee, memo, funds);
        });
        this.tick = (fee = "auto", memo, funds) => __awaiter(this, void 0, void 0, function* () {
            return yield this.client.execute(this.sender, this.contractAddress, {
                tick: {}
            }, fee, memo, funds);
        });
        this.client = client;
        this.sender = sender;
        this.contractAddress = contractAddress;
        this.updateSettings = this.updateSettings.bind(this);
        this.moveBalances = this.moveBalances.bind(this);
        this.registerAgent = this.registerAgent.bind(this);
        this.updateAgent = this.updateAgent.bind(this);
        this.checkInAgent = this.checkInAgent.bind(this);
        this.unregisterAgent = this.unregisterAgent.bind(this);
        this.withdrawReward = this.withdrawReward.bind(this);
        this.createTask = this.createTask.bind(this);
        this.removeTask = this.removeTask.bind(this);
        this.refillTaskBalance = this.refillTaskBalance.bind(this);
        this.refillTaskCw20Balance = this.refillTaskCw20Balance.bind(this);
        this.proxyCall = this.proxyCall.bind(this);
        this.receive = this.receive.bind(this);
        this.withdrawWalletBalance = this.withdrawWalletBalance.bind(this);
        this.tick = this.tick.bind(this);
    }
}
exports.CwCroncatCoreClient = CwCroncatCoreClient;
