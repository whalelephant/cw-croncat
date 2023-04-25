import { config } from "dotenv"
import { calculateFee } from "@cosmjs/stargate"
import * as fs from "fs"
import { getChainForAccount, getChainByChainName } from './utils'
import { DeploySigner } from "./signer"
import { FactoryClient } from './factory';
config({ path: '.env' })
const artifactsRoot = `${process.cwd()}/../../artifacts`

import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'

// CMD: yarn whitelist stars1b4kls73st8k5flxkwjyr4dfa3rwqtfary7ku86
// OR:
// CMD: yarn whitelist stars1b4kls73st8k5flxkwjyr4dfa3rwqtfary7ku86 stargazetestnet
const start = async () => {
  const args = yargs(hideBin(process.argv)).argv
  if (!args._ || args._.length <= 0) {
    console.error("Must specify an address to whitelist, use the command: 'yarn whitelist stars1v406awlqrx7tftjqsgsvy4pjcrnjraple3puf2'")
    process.exit()
  }
  const whitelistAddr = args._[0]
  console.info(`Adding ${whitelistAddr} to whitelisted agents...`)

	let chainName
	let network
	if (args._ && args._.length > 1) {
		chainName = args._[1]
		const chain = getChainByChainName(chainName)
		if (chain) {
			network = chain
		} else {
			console.error(`Couldn't find ${chainName}, please try different chain_name and try again.`)
			process.exit()
		}
	} else {
		network = getChainForAccount(whitelistAddr)
	}

	// Get the network client based on prefix from address
	const ds = new DeploySigner()
	const cwClient = await ds.init(network)
	const rawDeployed = fs.readFileSync(`${artifactsRoot}/${cwClient.chain.chain_name}-deployed_contracts.json`, 'utf8')
	if (!rawDeployed) {
		console.error(`No deployed contracts found for ${cwClient.chain.pretty_name}`)
		process.exit(1)
	}
	const deployedContracts = JSON.parse(rawDeployed)
	const contracts: any = {}
	deployedContracts.forEach(d => {
		// create a map instead of array
		contracts[d.name] = { codeId: d.code_id, address: d.address }
	})

	// Classes
	const factoryClient = new FactoryClient(cwClient, contracts.factory.address);

	// Pre-logic: get latest versions from factory
	const allVersions: any[] = await factoryClient.getLatestContracts()
	const versions: any = {}
	allVersions.forEach((v: any) => {
		// create a map instead of array
		versions[v.contract_name] = v.metadata
	})
	// console.log('factory allVersions', JSON.stringify(allVersions));

	// Add first agent Register & check status
	try {
		const r = await factoryClient.addWhitelistedAgent(
			versions.agents.contract_addr,
      whitelistAddr,
			calculateFee(555_000, cwClient.defaultGasPrice)
		);
		console.info(`Agents Add to Whitelist on ${cwClient.chain.pretty_name} SUCCESS\n`, JSON.stringify(r), '\n')
	} catch (e) {
		console.info(`Agents Add to Whitelist on ${cwClient.chain.pretty_name} ERROR`, e)
	}

	process.exit()
}

// Start deployment
(() => start())()
