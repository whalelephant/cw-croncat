import { calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
config({ path: '.env' })
import { getGitHash, getChecksums, getContractVersionFromCargoToml } from './utils'
import { DeploySigner } from "./signer"

export class ModulesClient {
  client: DeploySigner;
  uploadGas: any;
  executeGas: any;
  codeId: number;

  constructor(client: DeploySigner) {
    this.client = client;
  }

  async deploy(
    artifactsRoot: string,
    factoryAddress: string,
  ): Promise<any[]> {
    if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
    this.uploadGas = calculateFee(4_400_000, this.client.defaultGasPrice)
    this.executeGas = calculateFee(555_000, this.client.defaultGasPrice)
		const wasms = [
      fs.readFileSync(`${artifactsRoot}/croncat_mod_balances.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_dao.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_generic.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_nft.wasm`),
    ]

    const upload0 = await this.client.client.upload(this.client.accounts.deployer, wasms[0], this.uploadGas)
    const upload1 = await this.client.client.upload(this.client.accounts.deployer, wasms[1], this.uploadGas)
    const upload2 = await this.client.client.upload(this.client.accounts.deployer, wasms[2], this.uploadGas)
    const upload3 = await this.client.client.upload(this.client.accounts.deployer, wasms[3], this.uploadGas)
    const checksums = await getChecksums()
    const githash = await getGitHash()

    // get the version from cargo
    const version0 = await getContractVersionFromCargoToml('mod-balances')
    const version1 = await getContractVersionFromCargoToml('mod-dao')
    const version2 = await getContractVersionFromCargoToml('mod-generic')
    const version3 = await getContractVersionFromCargoToml('mod-nft')

    const exec0 = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload0.codeId,
          "version": version0,
          "commit_id": githash || '-',
          "checksum": checksums.mod_balances || '-',
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: `${version0[0]}.${version0[1]}` })).toString('base64'),
          "contract_name": "mod_balances"
        }
      }
    }, this.executeGas)

    const exec1 = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload1.codeId,
          "version": version1,
          "commit_id": githash || '-',
          "checksum": checksums.mod_dao || '-',
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: `${version1[0]}.${version1[1]}` })).toString('base64'),
          "contract_name": "mod_dao"
        }
      }
    }, this.executeGas)

    const exec2 = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload2.codeId,
          "version": version2,
          "commit_id": githash || '-',
          "checksum": checksums.mod_generic || '-',
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: `${version2[0]}.${version2[1]}` })).toString('base64'),
          "contract_name": "mod_generic"
        }
      }
    }, this.executeGas)

    const exec3 = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload3.codeId,
          "version": version3,
          "commit_id": githash || '-',
          "checksum": checksums.mod_nft || '-',
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: `${version3[0]}.${version3[1]}` })).toString('base64'),
          "contract_name": "mod_nft"
        }
      }
    }, this.executeGas)

    return [
      { name: 'mod_balances', code_id: upload0.codeId, address: exec0.logs[0].events[1].attributes[0].value },
      { name: 'mod_dao', code_id: upload1.codeId, address: exec1.logs[0].events[1].attributes[0].value },
      { name: 'mod_generic', code_id: upload2.codeId, address: exec2.logs[0].events[1].attributes[0].value },
      { name: 'mod_nft', code_id: upload3.codeId, address: exec3.logs[0].events[1].attributes[0].value },
    ];
  }
}
