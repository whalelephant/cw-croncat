import { ExecuteResult, toBinary } from "@cosmjs/cosmwasm-stargate";
import { QueryClient } from "@cosmjs/stargate";
import { StdFee, calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
import { getContractVersionFromCargoToml } from './utils'
import { DeploySigner } from "./signer"
config({ path: '.env' })

export class FactoryClient {
  client: DeploySigner;
  querier: any;
  uploadGas: any;
  executeGas: any;
  codeId: number;
  address: string;

  constructor(client: DeploySigner, querier?: QueryClient) {
		this.client = client;
    this.querier = querier || client.querier;
	}

	async deploy(
    artifactsRoot: string,
  ): Promise<[number, string]> {
    // Gas vals
    this.uploadGas = calculateFee(4_400_000, this.client.defaultGasPrice)
    this.executeGas = calculateFee(555_000, this.client.defaultGasPrice)
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_factory.wasm`)
    const uploadRes = await this.client.client.upload(this.client.accounts.deployer, wasm, this.uploadGas)
		this.codeId = uploadRes.codeId

    // get the version from cargo
    const version = await getContractVersionFromCargoToml('croncat-factory')

    const instantiateOptions = {
      admin: this.client.accounts.deployer,
      // memo: '',
      // funds: [],
    }

    // instantiate
    const instantiateGas = calculateFee(700_000, this.client.defaultGasPrice)
    const factoryInst = await this.client.client.instantiate(this.client.accounts.deployer, this.codeId, {}, `CronCat:factory:${version}`, instantiateGas, instantiateOptions)
    this.address = factoryInst.contractAddress

    return [this.codeId, this.address];
	}

	async getLatestContracts(): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { latest_contracts: {} };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
		return response;
	}

  async getLatestContract(contractName: string): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { latest_contract: { contract_name: contractName } };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
		return response;
	}

  async getVersionsByContractName(contractName: string): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { versions_by_contract_name: { contract_name: contractName } };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
		return response;
  }

  async getContractNames(): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { contract_names: {} };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
    return response;
  }

  async getAllEntries(): Promise<any> {
    if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
    const q = { all_entries: {} };
    const response = await this.querier.wasm.queryContractSmart(this.address, q);
    return response;
  }

  async doProxyCall(
    contractAddr: string,
    sub_msg: any,
    gas: StdFee,
    funds: any,
  ): Promise<ExecuteResult> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    const proxy_msg = {
      proxy: {
        msg: {
          execute: {
            contract_addr: contractAddr || '',
            msg: toBinary(sub_msg),
            funds: funds || [],
          }
        },
      }
    };
    const response = await this.client.client.execute(this.client.accounts.deployer, this.address, proxy_msg, gas, null, funds);
    return response;
  }

  async addWhitelistedAgent(
    agentContractAddr: string,
    agentAddr: string,
    gas: StdFee
  ): Promise<ExecuteResult> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    const proxy_sub_msg = {
      execute: {
        contract_addr: agentContractAddr || '',
        msg: toBinary({
          add_agent_to_whitelist: {
            agent_address: agentAddr || ''
          }
        }),
        funds: [],
      }
    }
    const proxy_msg = {
      proxy: {
        msg: proxy_sub_msg,
      }
    };
    const response = await this.client.client.execute(this.client.accounts.deployer, this.address, proxy_msg, gas);
    return response;
  }
}