import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'
import { coins } from "@cosmjs/proto-signing"
import { config } from "dotenv"
import * as fs from "fs"
import { calculateFee } from "@cosmjs/stargate"
import { getChainByChainName, getSupportedNetworks } from './utils'
import { DeploySigner } from "./signer"
import { FactoryClient } from './factory';
import { ManagerClient } from './manager';
import { TaskClient } from './tasks';
import { AgentClient } from './agents';
import { ModulesClient } from './modules';
config({ path: '.env' })
const artifactsRoot = `${process.cwd()}/../../artifacts`

const e2eTasks = async (cwClient) => {
	console.info(`🏖️ Starting ${cwClient.chain.pretty_name} End 2 End Task Variants 🌋`)

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

	// TODO: bring back
  // // Get blockchain status
  // let currentBlockHeight
  // try {
  //   const r = await tmClient.status()
  //   if (r?.syncInfo?.latestBlockHeight) currentBlockHeight = r.syncInfo.latestBlockHeight
  //   console.log('Current Block Height', currentBlockHeight);
  //   if (!currentBlockHeight) process.exit(1)
  // } catch (e) {
  //   console.info(`Blockchain Status ERROR`, e)
  //   process.exit(1)
  // }

	// Classes
	const factoryClient = new FactoryClient(cwClient);
	const managerClient = new ManagerClient(cwClient);
	const agentClient = new AgentClient(cwClient);
	const taskClient = new TaskClient(cwClient);
	// NOTE: Unsure if we really need module thangs here. maybe someday when haz too much hands and excessive timez

	// Pre-logic: get latest versions from factory
	const allVersions: any[] = await factoryClient.getLatestContracts()
	const versions: any = {}
	allVersions.forEach((v: any) => {
		// create a map instead of array
		versions[v.contract_name] = v.metadata
	})
	// console.log('factory allVersions', JSON.stringify(allVersions));

  // TODO: 

	// // Register & check status
	// try {
	// 	const r = await agentClient.register(userAddress, versions.agents.contract_addr, executeGas);
	// 	console.info(`Agents Register SUCCESS\n`, JSON.stringify(r), '\n')
	// } catch (e) {
	// 	console.info(`Agents Register ERROR`, e)
	// }
	// try {
	// 	const as = await agentClient.status(userAddress, versions.agents.contract_addr);
	// 	console.info(`Agents Status\n`, as.agent, '\n')
	// 	if (as.agent.status !== 'active') process.exit(1)
	// } catch (e) {
	// 	console.info(`Agents Status ERROR`, e)
	// }

  // const allTasks = tasks({
  //   currentHeight: currentBlockHeight + 64, // because it could take 64 blocks to create all these taskoids
  //   address: userAddress,
  //   amount: 1337,
  //   denom,
  // })

  // // console.log('tasks INTERVAL', JSON.stringify(allTasks.intervalTasks));
  // console.log('tasks INTERVAL', allTasks.intervalTasks.length);

	// // Create all tasks
  // // Loop all the intervals & create tasks
  // for await (const task of allTasks.intervalTasks) {
  //   try {
  //     console.log('TASK:', JSON.stringify(task));
  //     const t1 = await taskClient.create(userAddress, versions.tasks.contract_addr, executeGas, task, coins(250_000, denom));
  //     const task_hash = getTaskHashFromLogs(t1)
  //     console.info(`Task Create SUCCESS:`, task_hash, JSON.stringify(task.interval), JSON.stringify(task.boundary))
  //   } catch (e) {
  //     console.info(`Task Create ERROR`, e)
  //   }
	// }

	// // 1st agent do proxycall
	// try {
	// 	const a1pc = await managerClient.proxyCall(userAddress, versions.manager.contract_addr, executeGas);
	// 	console.info(`Agent 1 ProxyCall\n`, JSON.stringify(a1pc), '\n')
	// } catch (e) {
	// 	console.info(`Agent 1 ProxyCall ERROR`, e)
	// }

	// // 1st agent unregister
	// try {
	// 	const as1u = await agentClient.unregister(userAddress, versions.agents.contract_addr, executeGas);
	// 	console.info(`Agent 1 Unregister\n`, JSON.stringify(as1u), '\n')
	// } catch (e) {
	// 	console.info(`Agent 1 Unregister ERROR`, e)
	// }

	// Get list of all tasks
	let tasksFound = []
	try {
		const t = await taskClient.getTasks();
		console.info(`Tasks`, t.length)
		if (t.length <= 0) process.exit(1)
    tasksFound = t
	} catch (e) {
		console.info(`Tasks List ERROR`, e)
	}

	// Loop and remove all tasks
  for await (const task of tasksFound) {
		try {
			const t = await taskClient.remove(task.owner_addr, executeGas, task.task_hash);
			console.info(`Task Remove SUCCESS\n`, task.task_hash, '\n', JSON.stringify(t), '\n')
		} catch (e) {
			console.info(`Task Remove ERROR`, e)
		}
	}

	process.exit()
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
	Object.keys(networkClients).forEach(k => p.push(e2eTasks(networkClients[k])))
	await Promise.all(p)

	process.exit()
}

// Start deployment
(() => start())()
