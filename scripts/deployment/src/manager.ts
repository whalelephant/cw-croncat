import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
config({ path: '.env' })
import { getGitHash, getChecksums } from './utils'
const denom: string = process.env.DENOM

export class ManagerClient {
	client: SigningCosmWasmClient;

	constructor(client: SigningCosmWasmClient) {
		this.client = client;
	}

	async deploy(artifactsRoot: string, sender: string, factoryAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_manager.wasm`)
		const uploadRes = await this.client.upload(sender, wasm, uploadGas)
		const codeId = uploadRes.codeId
		const checksums = await getChecksums()
		const githash = await getGitHash()

		let base64ManagerInst = Buffer.from(JSON.stringify({
			"denom": denom,
			"owner_addr": sender,
			"croncat_tasks_key": ["tasks", [0, 1]],
			"croncat_agents_key": ["agents", [0, 1]]
		})).toString('base64')

		// instantiate manager contract (from the factory)
		const deployMsg = {
			"deploy": {
				"kind": "manager",
				"module_instantiate_info": {
					"code_id": codeId,
					"version": [0, 1],
					"commit_id": githash,
					"checksum": "nosleeptilsecurityaudit",
					"changelog_url": "https://github.com/croncats",
					"schema": "",
					"msg": base64ManagerInst,
					"contract_name": "manager"
				}
			}
		}

		const instRes = await this.client.execute(sender, factoryAddress, deployMsg, executeGas);
		const address: string = instRes.logs[0].events[1].attributes[0].value

		return [codeId, address];
	}

	async proxyCall(sender: string, contractAddr: string, gas: StdFee, task_hash?: any): Promise<ExecuteResult> {
		const msg = { proxy_call: { task_hash } };
		const response = await this.client.execute(sender, contractAddr, msg, gas);
		return response;
	}

	async tick(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
		const msg = { tick: {} };
		const response = await this.client.execute(sender, contractAddr, msg, gas);
		return response;
	}

	async ownerWithdraw(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
		const msg = { owner_withdraw: {} };
		const response = await this.client.execute(sender, contractAddr, msg, gas);
		return response;
	}

	async userWithdraw(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
		const msg = { user_withdraw: {} };
		const response = await this.client.execute(sender, contractAddr, msg, gas);
		return response;
	}

	async refillTaskBalance(sender: string, contractAddr: string, gas: StdFee, task_hash: any, funds: string): Promise<ExecuteResult> {
		const msg = { refill_task_balance: { task_hash } };
		const response = await this.client.execute(sender, contractAddr, msg, gas, funds);
		return response;
	}
}