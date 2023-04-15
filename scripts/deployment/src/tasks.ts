import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee, QueryClient, calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
config({ path: '.env' })
import { getGitHash, getChecksums, getContractVersionFromCargoToml, getInstantiatedAddrFromLogs } from './utils'
import { DeploySigner } from "./signer"

export class TaskClient {
  client: DeploySigner;
  querier: any;
  uploadGas: any;
  executeGas: any;
  codeId: number;
  address: string;

  constructor(client: DeploySigner, address?: string, querier?: QueryClient) {
    this.client = client;
    this.querier = querier || client.querier;

    if (address) this.address = address;
  }

  async deploy(
    artifactsRoot: string,
    factoryAddress: string,
  ): Promise<[number, string]> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    this.uploadGas = calculateFee(4_400_000, this.client.defaultGasPrice)
    this.executeGas = calculateFee(555_000, this.client.defaultGasPrice)
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_tasks.wasm`)
    const uploadRes = await this.client.client.upload(this.client.accounts.deployer, wasm, this.uploadGas)
    this.codeId = uploadRes.codeId

    const checksums = await getChecksums()
    const githash = await getGitHash()

    // get the version from cargo
    const version = await getContractVersionFromCargoToml('croncat-tasks')

    // instantiate manager contract (from the factory)
    const deployMsg = {
      "deploy": {
        "kind": "tasks",
        "module_instantiate_info": {
          "code_id": this.codeId,
          "version": version,
          "commit_id": githash || '-',
          "checksum": checksums.tasks || '-',
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({
            chain_name: this.client.prefix || 'juno',
            version: `${version[0]}.${version[1]}`,
            pause_admin: `${this.client.accounts.pause_admin}`,
            croncat_manager_key: ['manager', version || [0, 1]],
            croncat_agents_key: ['agents', version || [0, 1]],
            // slot_granularity_time: '',
            // gas_base_fee: '',
            // gas_action_fee: '',
            // gas_query_fee: '',
            // gas_limit: '',
          })).toString('base64'),
          "contract_name": "tasks"
        }
      }
    }
    const instRes = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, deployMsg, this.executeGas);
    // Get the first instantiated address
		this.address = getInstantiatedAddrFromLogs(instRes.logs)

    return [this.codeId, this.address];
  }

  async getTasks(): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { tasks: {} };
    // const q = { tasks_with_queries: {} };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
    return response;
  }

  async create(sender: string, gas: StdFee, task: any, funds: Coin[]): Promise<ExecuteResult> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    const msg = { create_task: { task } };
    const response = await this.client.client.execute(sender, this.address, msg, gas, undefined, funds);
    return response;
  }

  async remove(sender: string, gas: StdFee, task_hash: any): Promise<ExecuteResult> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    const msg = { remove_task: { task_hash } };
    const response = await this.client.client.execute(sender, this.address, msg, gas);
    return response;
  }
}
