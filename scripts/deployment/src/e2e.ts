import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'
import { coins } from "@cosmjs/proto-signing"
import { config } from "dotenv"
import * as fs from "fs"
import { calculateFee } from "@cosmjs/stargate"
import { getChainByChainName, getSupportedNetworks, sleep } from './utils'
import { DeploySigner } from "./signer"
import { FactoryClient } from './factory';
import { ManagerClient } from './manager';
import { TaskClient } from './tasks';
import { AgentClient } from './agents';
import { ModulesClient } from './modules';
config({ path: '.env' })
const artifactsRoot = `${process.cwd()}/../../artifacts`

const e2e = async (cwClient) => {
	console.info(`🏖️ Starting ${cwClient.chain.pretty_name} End 2 End Chex 🌋`)

	// Gas vals
	const executeGas = calculateFee(999_000, cwClient.defaultGasPrice)

	const rawDeployed = fs.readFileSync(`${artifactsRoot}/${cwClient.chain.chain_name}-deployed_contracts.json`, 'utf8')
	if (!rawDeployed) process.exit(1)
	const deployedContracts = JSON.parse(rawDeployed)
	const contracts: any = {}
	deployedContracts.forEach(d => {
		// create a map instead of array
		contracts[d.name] = { codeId: d.code_id, address: d.address }
	})
	if (!contracts?.factory?.address) {
		console.error(`No deployed factory found for ${cwClient.chain.pretty_name}`)
		process.exit()
	}

	// Classes
	const factoryClient = new FactoryClient(cwClient, contracts.factory.address);
	// NOTE: Unsure if we really need module thangs here. maybe someday when haz too much hands and excessive timez

	// Pre-logic: get latest versions from factory
	// NOTE: Could use the contracts object above, but def wanna be overly same as production
	let allVersions: any[]
	try {
		allVersions = await factoryClient.getLatestContracts()
	} catch (e) {
		console.log('factory allVersions error', e);
	}
	const versions: any = {}
	allVersions.forEach((v: any) => {
		// create a map instead of array
		versions[v.contract_name] = v.metadata
	})

	if (!versions?.manager?.contract_addr || !versions?.agents?.contract_addr || !versions?.tasks?.contract_addr) {
		console.error(`Missing deployed contracts for ${cwClient.chain.pretty_name}, try cmd 'yarn go ${cwClient.chain.chain_name}' again!`)
		process.exit()
	}

	const managerClient = new ManagerClient(cwClient, versions.manager.contract_addr);
	const agentClient = new AgentClient(cwClient, versions.agents.contract_addr);
	const taskClient = new TaskClient(cwClient, versions.tasks.contract_addr);

	// TODO: Replace with better example
	// const task1 = {
	// 	"actions": [
	// 		{
	// 			"msg": {
	// 				"wasm": {
	// 					"execute": {
	// 						"contract_addr": versions.manager.contract_addr,
	// 						"msg": Buffer.from(JSON.stringify({ "tick": {} })).toString('base64'),
	// 						"funds": []
	// 					}
	// 				}
	// 			},
	// 			"gas_limit": 75000
	// 		}
	// 	],
	// 	"boundary": null,
	// 	"cw20": null,
	// 	"interval": {
	// 		"block": 1
	// 	},
	// 	"stop_on_fail": true,
	// 	"queries": null,
	// 	"transforms": null
	// }
	const task2 = (amount: number) => ({
		"actions": [
			{
				"msg": {
					"bank": {
						"send": {
							"to_address": versions.manager.contract_addr,
							"amount": coins(amount, cwClient.fee_token.denom)
						}
					}
				},
				"gas_limit": 75000
			}
		],
		"boundary": null,
		"cw20": null,
		"interval": {
			"block": 1
		},
		"stop_on_fail": false,
		"queries": null,
		"transforms": null
	})

	const factoryTask1 = {
		"create_task": {
			"task": {
				"actions": [
					{
						"msg": {
							"wasm": {
								"execute": {
									"contract_addr": versions.agents.contract_addr,
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
					"block": 1
				},
				"stop_on_fail": true,
				"queries": null,
				"transforms": null
			}
		}
	}

	// NOTE: Only uncomment if needed!
	// // Add 3rd agent to whitelist
	// try {
	// 	const r = await factoryClient.addWhitelistedAgent(
	// 		versions.agents.contract_addr,
	// 		cwClient.accounts.agent3,
	// 		executeGas
	// 	);
	// 	console.info(`Agents Add to Whitelist SUCCESS\n`, JSON.stringify(r), '\n')
	// } catch (e) {
	// 	console.info(`Agents Add to Whitelist ERROR`, e)
	// }

	// Register & check status
	try {
		const r = await agentClient.register(cwClient.accounts.agent1, executeGas);
		console.info(`Agents Register SUCCESS\n`, JSON.stringify(r), '\n')
	} catch (e) {
		console.info(`Agents Register ERROR`, e)
	}
	try {
		const as = await agentClient.status(cwClient.accounts.agent1);
		console.info(`Agents Status\n`, as.agent, '\n')
		if (as.agent.status !== 'active') process.exit(1)
	} catch (e) {
		console.info(`Agents Status ERROR`, e)
	}

	// Create 2 tasks (first one, the stock factory tick task)
	try {
		const t1 = await factoryClient.doProxyCall(
			versions.tasks.contract_addr,
			factoryTask1,
			executeGas,
			coins(60_000, cwClient.fee_token.denom)
		);
		console.info(`Factory Task 1 Create SUCCESS\n`, JSON.stringify(t1), '\n')
	} catch (e) {
		console.info(`Factory Task 1 Create ERROR`, e)
	}
	// TODO: Bring back once better example above
	// try {
	// 	const t1 = await taskClient.create(cwClient.accounts.deployer, executeGas, task1, coins(100_000, cwClient.fee_token.denom));
	// 	console.info(`Task 1 Create SUCCESS\n`, JSON.stringify(t1), '\n')
	// } catch (e) {
	// 	console.info(`Task 1 Create ERROR`, e)
	// }
	try {
		const t2 = await taskClient.create(cwClient.accounts.deployer, executeGas, task2(1), coins(100_000, cwClient.fee_token.denom));
		console.info(`Task 2 Create SUCCESS\n`, JSON.stringify(t2), '\n')
	} catch (e) {
		console.info(`Task 2 Create ERROR`, e)
	}

	// Register 2nd agent & check status
	try {
		const r2 = await agentClient.register(cwClient.accounts.agent2, executeGas);
		console.info(`Agents Register SUCCESS\n`, JSON.stringify(r2), '\n')
	} catch (e) {
		console.info(`Agents Register ERROR`, e)
	}
	try {
		const as2 = await agentClient.status(cwClient.accounts.agent2);
		console.info(`Agent 2 Status\n`, as2.agent, '\n')
		if (as2.agent.status !== 'pending') process.exit(1)
	} catch (e) {
		console.info(`Agent 2 Status ERROR`, e)
	}

	// create another 2 tasks so second agent can be nominated
	try {
		const t3 = await taskClient.create(cwClient.accounts.deployer, executeGas, task2(2), coins(260_000, cwClient.fee_token.denom));
		console.info(`Task 3 Create SUCCESS\n`, JSON.stringify(t3), '\n')
	} catch (e) {
		console.info(`Task 3 Create ERROR`, e)
	}
	try {
		const t4 = await taskClient.create(cwClient.accounts.deployer, executeGas, task2(3), coins(460_000, cwClient.fee_token.denom));
		console.info(`Task 4 Create SUCCESS\n`, JSON.stringify(t4), '\n')
	} catch (e) {
		console.info(`Task 4 Create ERROR`, e)
	}

	// Sleep a couple blocks, because for whatever reason, we need chain sync'd for sure
	await sleep(12 * 1000);

	// confirm agent is nominated
	try {
		const as2 = await agentClient.status(cwClient.accounts.agent2);
		console.info(`Agent 2 Nominated`, as2.agent.status)
		if (as2.agent.status !== 'nominated') {
			console.info(`Agent 2 Nominated Still pending, should be nominated!`)
			process.exit(1)
		}
	} catch (e) {
		console.info(`Agent 2 Nominated ERROR`, e)
	}

	// Check in 2nd agent
	try {
		const as2 = await agentClient.checkIn(cwClient.accounts.agent2, executeGas);
		console.info(`Agent 2 Checkin\n`, JSON.stringify(as2), '\n')
	} catch (e) {
		console.info(`Agent 2 Nominated ERROR`, e)
	}
	// confirm agent is active
	try {
		const as2 = await agentClient.status(cwClient.accounts.agent2);
		console.info(`Agent 2 Active`, as2.agent.status)
		if (as2.agent.status !== 'active') process.exit(1)
	} catch (e) {
		console.info(`Agent 2 Activated ERROR`, e)
	}

	// 1st agent do proxycall
	try {
		const a1pc = await managerClient.proxyCall(cwClient.accounts.agent1, executeGas);
		console.info(`Agent 1 ProxyCall\n`, JSON.stringify(a1pc), '\n')
	} catch (e) {
		console.info(`Agent 1 ProxyCall ERROR`, e)
	}

	// 2nd agent do proxycall
	try {
		const a2pc = await managerClient.proxyCall(cwClient.accounts.agent2, executeGas);
		console.info(`Agent 2 ProxyCall\n`, JSON.stringify(a2pc), '\n')
	} catch (e) {
		console.info(`Agent 2 ProxyCall ERROR`, e)
	}

	// 1st agent withdraw reward
	try {
		const a1w = await managerClient.agentWithdraw(cwClient.accounts.agent1, executeGas);
		console.info(`Agent 1 Withdraw\n`, JSON.stringify(a1w), '\n')
	} catch (e) {
		console.info(`Agent 1 Withdraw ERROR`, e)
	}

	// 1st agent unregister
	try {
		const as1u = await agentClient.unregister(cwClient.accounts.agent1, executeGas);
		console.info(`Agent 1 Unregister\n`, JSON.stringify(as1u), '\n')
	} catch (e) {
		console.info(`Agent 1 Unregister ERROR`, e)
	}

	// 2nd agent unregister
	try {
		const as2u = await agentClient.unregister(cwClient.accounts.agent2, executeGas);
		console.info(`Agent 2 Unregister\n`, JSON.stringify(as2u), '\n')
	} catch (e) {
		console.info(`Agent 2 Unregister ERROR`, e)
	}

	// Confirm no agents
	try {
		const aIds = await agentClient.getAgents();
		console.info(`Agents List Empty`, aIds)
		if (aIds.active.length > 0 || aIds.pending.length > 0) process.exit(1)
	} catch (e) {
		console.info(`Agents List Empty ERROR`, e)
	}

	// Get list of all tasks
	let tasks = []
	try {
		const t = await taskClient.getTasks();
		console.info(`Tasks`, t.length)
		if (t.length <= 0) process.exit(1)
		tasks = t
	} catch (e) {
		console.info(`Tasks List ERROR`, e)
	}

	// Loop and remove all tasks
	for await (const task of tasks) {
		try {
			// remove all tasks except the tick from factory!!
			if (task.owner_addr != contracts.factory.address) {
				const t = await taskClient.remove(task.owner_addr, executeGas, task.task_hash);
				console.info(`Task Remove SUCCESS\n`, task.task_hash, '\n', JSON.stringify(t), '\n')
			}
		} catch (e) {
			console.info(`Task Remove ERROR`, e)
		}
	}
}

// Bootstrap all networks before deploying contexts
const start = async () => {
	const args = yargs(hideBin(process.argv)).argv
	let chainName
	let networks = []
	if (args._ && args._.length > 0) {
		chainName = args._[0]
		const chain = getChainByChainName(chainName)
		if (chain) {
			networks = [chain]
		} else {
			console.error(`Couldn't find ${chainName}, please try different chain_name and try again.`)
			process.exit()
		}
	} else {
		networks = getSupportedNetworks()
	}
	if (!networks || !networks.length) process.exit();
	// Instantiate all the clients needed
	const networkClients = {}
	try {
		await Promise.all(networks.map(async n => {
			const ds = new DeploySigner()
			networkClients[n.chain_name] = await ds.init(n)
			return n
		}))
	} catch (e) {
		console.log(e);
	}

	if (!Object.keys(networkClients) || !Object.keys(networkClients).length) process.exit()

	// loop all clients and display their address/balances
	const p = []
	Object.keys(networkClients).forEach(k => p.push(e2e(networkClients[k])))
	await Promise.all(p)

	process.exit()
}

// Start deployment
(() => start())()
