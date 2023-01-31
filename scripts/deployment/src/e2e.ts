import { setupWasmExtension, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { coins, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { HttpBatchClient, Tendermint34Client, TxResponse } from "@cosmjs/tendermint-rpc"
import { QueryClient } from "@cosmjs/stargate";
import { fromHex } from "@cosmjs/encoding";
import { config } from "dotenv"
import { GasPrice, StdFee, calculateFee } from "@cosmjs/stargate"
import * as fs from "fs"
import * as util from "util"
import { FactoryClient } from './factory';
import { ManagerClient } from './manager';
import { TaskClient } from './tasks';
import { AgentClient } from './agents';
import { ModulesClient } from './modules';
config({ path: '.env' })
// Get values from the environment variables located in the .env file
const seedPhrase: string = process.env.SEED_PHRASE
const prefix: string = process.env.PREFIX
const endpoint: string = process.env.RPC_ENDPOINT
const denom: string = process.env.DENOM
const defaultGasPrice = GasPrice.fromString(`0.025${denom}`)
const artifactsRoot = `${process.cwd()}/../../artifacts`

// Gas vals
const executeGas = calculateFee(555_000, defaultGasPrice)

const start = async () => {
	console.info(`🏖️ Starting End 2 End Chex 🌋`)

	const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, { prefix })
	const userAddress = (await signerWallet.getAccounts())[0].address
	const cwClient = await SigningCosmWasmClient.connectWithSigner(endpoint, signerWallet, { prefix, gasPrice: defaultGasPrice })
	const httpBatchClient = new HttpBatchClient(endpoint, {
			batchSizeLimit: 2,
			dispatchInterval: 500
	})
	const tmClient = await Tendermint34Client.create(httpBatchClient)
	const queryClient = QueryClient.withExtensions(tmClient, setupWasmExtension)
	const rawDeployed = fs.readFileSync(`${artifactsRoot}/${process.env.CHAIN_ID}_deployed_contracts.json`, 'utf8')
	if (!rawDeployed) process.exit(1)
	const deployedContracts = JSON.parse(rawDeployed)
	const contracts: any = {}
	deployedContracts.forEach(d => {
		// create a map instead of array
		contracts[d.name] = { codeId: d.code_id, address: d.address }
	})

	// Classes
	const factoryClient = new FactoryClient(cwClient, queryClient);
	const managerClient = new ManagerClient(cwClient);
	const agentClient = new AgentClient(cwClient, queryClient);
	const taskClient = new TaskClient(cwClient);
	// NOTE: Unsure if we really need module thangs here. maybe someday when haz too much hands and excessive timez

	// Pre-logic: get latest versions from factory
	const allVersions: any[] = await factoryClient.getLatestContracts(contracts.factory.address)
	const versions: any = {}
	allVersions.forEach((v: any) => {
		// create a map instead of array
		versions[v.contract_name] = v.metadata
	})
	// console.log('factory allVersions', JSON.stringify(allVersions));

	// TODO: Logic:
	// 1. register agent
	// 2. get status (active)
	// 3. create 2 tasks
	// 4. register agent
	// 5. get status (pending)
	// 6. create 2 tasks
	// 7. get status (nominated)
	// 8. check in
	// 9. proxy_call
	// 10. withdraw agent balance
	// 11. unregister


	// Register & check status
	try {
		const r = await agentClient.register(userAddress, versions.agents.contract_addr, executeGas);
		console.info(`Agents Register SUCCESS`, r)
	} catch (e) {
		console.info(`Agents Register ERROR`, e)
	}
	try {
		const as = await agentClient.status(userAddress, versions.agents.contract_addr);
		console.info(`Agents Status`, as.status)
		if (as.status !== 'active') process.exit(1)
	} catch (e) {
		console.info(`Agents Status ERROR`, e)
	}

	// Create 2 tasks
	try {
		const task = {
			"actions": [
				{
					"msg": {
						"wasm": {
							"execute": {
								"contract_addr": versions.manager.contract_addr,
								"msg": Buffer.from(JSON.stringify({ "tick": {} })).toString('base64'),
								"funds": []
							}
						}
					},
					"gas_limit": 75000
				}
			],
			"boundary": null,
			"cw20": null,
			"interval": {
				"block": 10
			},
			"stop_on_fail": true,
			"queries": null,
			"transforms": null
		}
		const t1 = await taskClient.create(userAddress, versions.tasks.contract_addr, executeGas, task, coins(60_000, denom));
		console.info(`Task 1 Create SUCCESS`, t1)
	} catch (e) {
		console.info(`Task 1 Create ERROR`, e)
	}

	process.exit()
}

// Start deployment
(() => start())()
