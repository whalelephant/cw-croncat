import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
import { getGitHash, getChecksums, getContractVersionFromCargoToml } from './utils'
config({ path: '.env' })
const prefix: string = process.env.PREFIX

export class ModulesClient {
  client: SigningCosmWasmClient;

  constructor(client: SigningCosmWasmClient) {
    this.client = client;
  }

  async deploy(artifactsRoot: string, sender: string, factoryAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<any[]> {
    const wasms = [
      fs.readFileSync(`${artifactsRoot}/croncat_mod_balances.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_dao.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_generic.wasm`),
      fs.readFileSync(`${artifactsRoot}/croncat_mod_nft.wasm`),
    ]

    const upload0 = await this.client.upload(sender, wasms[0], uploadGas)
    const upload1 = await this.client.upload(sender, wasms[1], uploadGas)
    const upload2 = await this.client.upload(sender, wasms[2], uploadGas)
    const upload3 = await this.client.upload(sender, wasms[3], uploadGas)
    const checksums = await getChecksums()
    const githash = await getGitHash()

    // get the version from cargo
    const version0 = await getContractVersionFromCargoToml('mod-balances')
    const version1 = await getContractVersionFromCargoToml('mod-dao')
    const version2 = await getContractVersionFromCargoToml('mod-generic')
    const version3 = await getContractVersionFromCargoToml('mod-nft')

    const exec0 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload0.codeId,
          "version": version0.split('.').slice(0, 2),
          "commit_id": githash,
          "checksum": checksums.mod_balances,
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: version0 })).toString('base64'),
          "contract_name": "mod_balances"
        }
      }
    }, executeGas)

    const exec1 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload1.codeId,
          "version": version1.split('.').slice(0, 2),
          "commit_id": githash,
          "checksum": checksums.mod_dao,
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: version1 })).toString('base64'),
          "contract_name": "mod_dao"
        }
      }
    }, executeGas)

    const exec2 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload2.codeId,
          "version": version2.split('.').slice(0, 2),
          "commit_id": githash,
          "checksum": checksums.mod_generic,
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: version2 })).toString('base64'),
          "contract_name": "mod_generic"
        }
      }
    }, executeGas)

    const exec3 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload3.codeId,
          "version": version3.split('.').slice(0, 2),
          "commit_id": githash,
          "checksum": checksums.mod_nft,
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({ version: version3 })).toString('base64'),
          "contract_name": "mod_nft"
        }
      }
    }, executeGas)

    return [
      { name: 'mod_balances', code_id: upload0.codeId, address: exec0.logs[0].events[1].attributes[0].value },
      { name: 'mod_dao', code_id: upload1.codeId, address: exec1.logs[0].events[1].attributes[0].value },
      { name: 'mod_generic', code_id: upload2.codeId, address: exec2.logs[0].events[1].attributes[0].value },
      { name: 'mod_nft', code_id: upload3.codeId, address: exec3.logs[0].events[1].attributes[0].value },
    ];
  }
}
