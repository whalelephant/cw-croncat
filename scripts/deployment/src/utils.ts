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