import { setupWasmExtension, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { coins, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { HdPath, stringToPath } from "@cosmjs/crypto"
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
import { tasks } from './taskSampleData';
import { getTaskHashFromLogs } from './utils'
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

	const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, {
		prefix,
		hdPaths: [
			// for easier coinage management
			stringToPath(`m/44'/118'/0'/0/0`),
			stringToPath(`m/44'/118'/0'/0/1`),
		]
	})
	const accts = await signerWallet.getAccounts()
	const userAddress = accts[0].address
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

  // Get blockchain status
  let currentBlockHeight
  try {
    const r = await tmClient.status()
    if (r?.syncInfo?.latestBlockHeight) currentBlockHeight = r.syncInfo.latestBlockHeight
    console.log('Current Block Height', currentBlockHeight);
    if (!currentBlockHeight) process.exit(1)
  } catch (e) {
    console.info(`Blockchain Status ERROR`, e)
    process.exit(1)
  }

	// Classes
	const factoryClient = new FactoryClient(cwClient, queryClient);
	const managerClient = new ManagerClient(cwClient);
	const agentClient = new AgentClient(cwClient, queryClient);
	const taskClient = new TaskClient(cwClient, queryClient);
	// NOTE: Unsure if we really need module thangs here. maybe someday when haz too much hands and excessive timez

	// Pre-logic: get latest versions from factory
	const allVersions: any[] = await factoryClient.getLatestContracts(contracts.factory.address)
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
		const t = await taskClient.getTasks(versions.tasks.contract_addr);
		console.info(`Tasks`, t.length)
		if (t.length <= 0) process.exit(1)
    tasksFound = t
	} catch (e) {
		console.info(`Tasks List ERROR`, e)
	}

	// Loop and remove all tasks
  for await (const task of tasksFound) {
		try {
			const t = await taskClient.remove(task.owner_addr, versions.tasks.contract_addr, executeGas, task.task_hash);
			console.info(`Task Remove SUCCESS\n`, task.task_hash, '\n', JSON.stringify(t), '\n')
		} catch (e) {
			console.info(`Task Remove ERROR`, e)
		}
	}

	process.exit()
}

// Start deployment
(() => start())()
