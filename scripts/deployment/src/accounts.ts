import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'
import { config } from "dotenv"
config({ path: '.env' })
import { getChainByChainName, getSupportedNetworks } from './utils'
import { DeploySigner } from "./signer"

// CMDs: 
// ```
// #  Display accounts for specific network or all networks (see .env for supported networks list)
// yarn accounts
// yarn accounts junotestnet
// ```
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
  Object.keys(networkClients).forEach(k => p.push(networkClients[k].listAccounts()))
  await Promise.all(p)

	process.exit()
}

// Start deployment
(() => start())()
