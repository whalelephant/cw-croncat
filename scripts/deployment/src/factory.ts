import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { QueryClient } from "@cosmjs/stargate";
import { GasPrice, StdFee, calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
import { getContractVersionFromCargoToml } from './utils'
import toml from "toml";
config({ path: '.env' })
const denom: string = process.env.DENOM
const defaultGasPrice = GasPrice.fromString(`0.025${denom}`)
const instantiateGas = calculateFee(700_000, defaultGasPrice)

export class FactoryClient {
	client: SigningCosmWasmClient;
  querier: any;

  constructor(client: SigningCosmWasmClient, querier?: QueryClient) {
		this.client = client;
    this.querier = querier;
	}

	async deploy(artifactsRoot: string, sender: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_factory.wasm`)
		const uploadRes = await this.client.upload(sender, wasm, uploadGas)
		const codeId = uploadRes.codeId

        // get the version from cargo
        let projectTomlPath = `${artifactsRoot}/../Cargo.toml`
        if (process.env.REGULAR === 'yes') projectTomlPath = `${artifactsRoot}/../../Cargo.toml`
        let crateToml = fs.readFileSync(projectTomlPath, 'utf8')
        const data = toml.parse(crateToml)

        const instantiateOptions = {
          admin: sender,
          // memo: '',
          // funds: [],
        }

        // get the version from cargo
        const version = await getContractVersionFromCargoToml('croncat-factory')
        // instantiate
        const factoryInst = await this.client.instantiate(sender, codeId, {}, `CronCat:factory:${version}`, instantiateGas, instantiateOptions)
        const address = factoryInst.contractAddress

		return [codeId, address];
	}

	async getLatestContracts(contractAddr: string): Promise<any> {
    const q = { latest_contracts: {} };
    const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
		return response;
	}

	async getLatestContract(contractAddr: string, contractName: string): Promise<any> {
    const q = { latest_contract: { contract_name: contractName } };
    const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
		return response;
	}

	async getVersionsByContractName(contractAddr: string, contractName: string): Promise<any> {
    const q = { versions_by_contract_name: { contract_name: contractName } };
    const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
		return response;
  }

  async getContractNames(contractAddr: string): Promise<any> {
    const q = { contract_names: {} };
    const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
    return response;
  }

  async getAllEntries(contractAddr: string): Promise<any> {
    const q = { all_entries: {} };
    const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
    return response;
  }
}