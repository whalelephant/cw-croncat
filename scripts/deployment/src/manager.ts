import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { StdFee, calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
config({ path: '.env' })
import { getContractVersionFromCargoToml, getGitHash, getChecksums, getInstantiatedAddrFromLogs } from './utils'
import { DeploySigner } from "./signer"

export class ManagerClient {
	client: DeploySigner;
	uploadGas: any;
	executeGas: any;
	codeId: number;
	address: string;

	constructor(client: DeploySigner, address?: string) {
		this.client = client;

		if (address) this.address = address;
	}

	async deploy(
		artifactsRoot: string,
		factoryAddress: string,
	): Promise<[number, string]> {
		if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
		this.uploadGas = calculateFee(4_400_000, this.client.defaultGasPrice)
		this.executeGas = calculateFee(555_000, this.client.defaultGasPrice)
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_manager.wasm`)
		const uploadRes = await this.client.client.upload(this.client.accounts.deployer, wasm, this.uploadGas)
		this.codeId = uploadRes.codeId

		const checksums = await getChecksums()
		const githash = await getGitHash()

		// get the version from cargo
		const version = await getContractVersionFromCargoToml('croncat-manager')

		let base64ManagerInst = Buffer.from(JSON.stringify({
			"version": `${version[0]}.${version[1]}`,
			"pause_admin": `${this.client.accounts.pause_admin}`,
			"treasury_addr": `${this.client.accounts.treasury}`,
			"croncat_tasks_key": ["tasks", version || [0, 1]],
			"croncat_agents_key": ["agents", version || [0, 1]]
		})).toString('base64')

		// instantiate manager contract (from the factory)
		const deployMsg = {
			"deploy": {
				"kind": "manager",
				"module_instantiate_info": {
					"code_id": this.codeId,
					"version": version,
					"commit_id": githash || '-',
					"checksum": checksums.manager || '-',
					"changelog_url": "https://github.com/croncats",
					"schema": "",
					"msg": base64ManagerInst,
					"contract_name": "manager"
				}
			}
		}

		// SUPER PANIC MODE if we dont have the denom to use
		if (!this.client.fee_token || !this.client.fee_token.denom) {
			return Promise.reject("Missing denom from fee_token.denom!")
		}

		const instRes = await this.client.client.execute(
			this.client.accounts.deployer,
			factoryAddress,
			deployMsg,
			this.executeGas,
			null,
			[{ "amount": "1", "denom": this.client.fee_token.denom }],
		);
		// Get the first instantiated address
		this.address = getInstantiatedAddrFromLogs(instRes.logs)

		return [this.codeId, this.address];
	}

	async proxyCall(sender: string, gas: StdFee, task_hash?: any): Promise<ExecuteResult> {
		if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
		const msg = { proxy_call: { task_hash } };
		const response = await this.client.client.execute(sender, this.address, msg, gas);
		return response;
	}

	// NOTE: This can only be done via factory!
	// async ownerWithdraw(sender: string, gas: StdFee): Promise<ExecuteResult> {
	// 	const msg = { owner_withdraw: {} };
	// 	const response = await this.client.client.execute(sender, this.address, msg, gas);
	// 	return response;
	// }

	async userWithdraw(sender: string, gas: StdFee): Promise<ExecuteResult> {
		if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
		const msg = { user_withdraw: {} };
		const response = await this.client.client.execute(sender, this.address, msg, gas);
		return response;
	}

	async agentWithdraw(sender: string, gas: StdFee): Promise<ExecuteResult> {
		if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
		const msg = { agent_withdraw: null };
		const response = await this.client.client.execute(sender, this.address, msg, gas);
		return response;
	}

	async refillTaskBalance(sender: string, gas: StdFee, task_hash: any, funds: string): Promise<ExecuteResult> {
		if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
		const msg = { refill_task_balance: { task_hash } };
		const response = await this.client.client.execute(sender, this.address, msg, gas, funds);
		return response;
	}
}