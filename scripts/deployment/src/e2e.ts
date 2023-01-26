import { setupWasmExtension, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
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

	// Pre-logic Checks
	// - get versions from factory
	// - get contract names from factory
	const allVersions = await factoryClient.getLatestContracts(contracts.factory.address)
	console.log('factory allVersions', allVersions);

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

	// // Agents
	// var agentClient = new AgentClient(cwClient);
	// // var [agentContractCodeId, agentContractAddr] = await agentClient.deploy(artifactsRoot, userAddress, factoryAddress, managerAddress, uploadGas, executeGas);
	// // console.info(`🏗️  Agents Done`)

	process.exit()
}

// Start deployment
(() => start())()
