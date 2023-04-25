import path from 'path'
import * as fs from "fs"
import toml from 'toml'
import { fromBech32 } from "@cosmjs/encoding";
import { chains } from "chain-registry"
import { config } from "dotenv"
config({ path: '.env' })
const artifactsRoot = `${process.cwd()}/../../artifacts`
const contractsRoot = `${process.cwd()}/../../contracts`

const networkType = process.env.NETWORK_TYPE || 'testnet'

// NOTE: MUST Be a contract wallet - multisig prefered!
// If you need one, go to https://github.com/CosmWasm/cw-plus/tree/main/contracts/cw3-fixed-multisig, compile, instantiate & get deployed address.
export const pauseAdmins = {
  anomatestnet: process.env.PAUSE_ADMIN_MULTISIG_ANOMATESTNET || '',
  anoma: process.env.PAUSE_ADMIN_MULTISIG_ANOMA || '',
  archwaytestnet: process.env.PAUSE_ADMIN_MULTISIG_ARCHWAYTESTNET || '',
  archway: process.env.PAUSE_ADMIN_MULTISIG_ARCHWAY || '',
  junotestnet: process.env.PAUSE_ADMIN_MULTISIG_JUNOTESTNET || '',
  juno: process.env.PAUSE_ADMIN_MULTISIG_JUNO || '',
  migalootestnet: process.env.PAUSE_ADMIN_MULTISIG_MIGALOOTESTNET || '',
  migaloo: process.env.PAUSE_ADMIN_MULTISIG_MIGALOO || '',
  namadatestnet: process.env.PAUSE_ADMIN_MULTISIG_NAMADATESTNET || '',
  namada: process.env.PAUSE_ADMIN_MULTISIG_NAMADA || '',
  neutrontestnet: process.env.PAUSE_ADMIN_MULTISIG_NEUTRONTESTNET || '',
  neutron: process.env.PAUSE_ADMIN_MULTISIG_NEUTRON || '',
  noistestnet: process.env.PAUSE_ADMIN_MULTISIG_NOISTESTNET || '',
  nois: process.env.PAUSE_ADMIN_MULTISIG_NOIS || '',
  osmosistestnet: process.env.PAUSE_ADMIN_MULTISIG_OSMOSISTESTNET || '',
  osmosistestnet5: process.env.PAUSE_ADMIN_MULTISIG_OSMOSISTESTNET5 || '',
  osmosis: process.env.PAUSE_ADMIN_MULTISIG_OSMOSIS || '',
  quasartestnet: process.env.PAUSE_ADMIN_MULTISIG_QUASARTESTNET || '',
  quasar: process.env.PAUSE_ADMIN_MULTISIG_QUASAR || '',
  seitestnet: process.env.PAUSE_ADMIN_MULTISIG_SEITESTNET || '',
  sei: process.env.PAUSE_ADMIN_MULTISIG_SEI || '',
  stargazetestnet: process.env.PAUSE_ADMIN_MULTISIG_STARGAZETESTNET || '',
  stargaze: process.env.PAUSE_ADMIN_MULTISIG_STARGAZE || '',
}

export const getSupportedNetworks = () => {
  // Get env list, then parse
  const chainNames = `${process.env.SUPPORTED_CHAIN_NAMES || ''}`.split(',')
  if (!chainNames || !chainNames.length) return []

  // Get chain registry for each one, if found
  return chainNames.map(cn => {
    return chains.find(c => c.chain_name === cn)
  }).filter(c => c != null)
}

export const getChainByChainName = cn => chains.find(c => c.chain_name === cn)

export const getChainForAccount = address => {
  const { prefix } = fromBech32(address);
  return chains.find(n => n?.bech32_prefix === prefix && n?.network_type === networkType);
}

export const getChecksums = async (): Promise<any> => {
  const sums = fs.readFileSync(`${artifactsRoot}/checksums.txt`, 'utf8')
  const lines = sums.split('\n')
  const m = {}
  lines.forEach(l => {
    const a = l.split('  ')
    const k = `${a[1]}`.replace('croncat_', '').split('.')[0]
    if (a.length > 1) m[k] = a[0]
  })
  return m  
}

export const getContractVersionFromCargoToml = async (contractName): Promise<any> => {
  const crateToml = fs.readFileSync(path.join(contractsRoot, contractName, 'Cargo.toml'), 'utf8')
  const data = toml.parse(crateToml)
  const sv = `${data.package.version || data.workspace.package.version}`.split('.').slice(0, 2)
  if (!sv || sv.length < 1) return [0, 0]
  return [parseInt(sv[0], 10), parseInt(sv[1], 10)]
}

export const getGitHash = () => {
  return new Promise((res, rej) => {
    require('child_process').exec('git rev-parse HEAD', function (err, stdout) {
      if (err) return rej(err)
      res(stdout)
    })
  })
}

export const getTaskHashFromLogs = (data: any) => {
  let task_hash

  data.events.forEach(e => {
    if (e.type === 'wasm') {
      e.attributes.forEach(a => {
        if (a.key === 'task_hash') task_hash = a.value
      })
    }
  })

  return task_hash
}

// Get the first instantiated address
export const getInstantiatedAddrFromLogs = logs => {
  let address

  logs.forEach(log => {
    log.events.forEach(e => {
      if (e.type === 'instantiate') {
        e.attributes.forEach(attr => {
          if (attr.key === '_contract_address' && attr.value && !address) address = attr.value
        })
      }
    })
  })

  return address
}

export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}