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
config({ path: '.env' })
// Get values from the environment variables located in the .env file
const seedPhrase: string = process.env.SEED_PHRASE
const prefix: string = process.env.PREFIX
const endpoint: string = process.env.RPC_ENDPOINT
// NOTE: MUST Be a contract wallet - multisig prefered!
// If you need one, go to https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw3-fixed-multisig, compile, instantiate & get deployed address.
const pauseAdminAddress: string = process.env.PAUSE_ADMIN_MULTISIG || ''
const denom: string = process.env.DENOM
const defaultGasPrice = GasPrice.fromString(`0.025${denom}`)
const artifactsRoot = `${process.cwd()}/../../artifacts`

import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'

// Gas vals
const executeGas = calculateFee(999_000, defaultGasPrice)

// CMD: yarn whitelist stars1b4kls73st8k5flxkwjyr4dfa3rwqtfary7ku86
const start = async () => {
  const args = yargs(hideBin(process.argv)).argv
  if (!args._ || args._.length <= 0) {
    console.error("Must specify an address to whitelist, use the command: 'yarn whitelist stars1v406awlqrx7tftjqsgsvy4pjcrnjraple3puf2'")
    process.exit()
  }
  const whitelistAddr = args._[0]
  console.info(`Adding ${whitelistAddr} to whitelisted agents...`)

	const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, {
		prefix,
		hdPaths: [
			// for easier coinage management
			stringToPath(`m/44'/118'/0'/0/0`),
			stringToPath(`m/44'/118'/0'/0/1`),
			stringToPath(`m/44'/118'/0'/0/2`),
			stringToPath(`m/44'/118'/0'/0/3`),
			stringToPath(`m/44'/118'/0'/0/4`),
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

	// Classes
	const factoryClient = new FactoryClient(cwClient, queryClient);

	// Pre-logic: get latest versions from factory
	const allVersions: any[] = await factoryClient.getLatestContracts(contracts.factory.address)
	const versions: any = {}
	allVersions.forEach((v: any) => {
		// create a map instead of array
		versions[v.contract_name] = v.metadata
	})
	// console.log('factory allVersions', JSON.stringify(allVersions));

	// Add first agent Register & check status
	try {
		const r = await factoryClient.addWhitelistedAgent(
			userAddress,
			contracts.factory.address,
			versions.agents.contract_addr,
      whitelistAddr,
			executeGas
		);
		console.info(`Agents Add to Whitelist SUCCESS\n`, JSON.stringify(r), '\n')
	} catch (e) {
		console.info(`Agents Add to Whitelist ERROR`, e)
	}

	process.exit()
}

// Start deployment
(() => start())()
