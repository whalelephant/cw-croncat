import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
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

    const initMsg = Buffer.from(JSON.stringify({})).toString('base64')

    const exec0 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload0.codeId,
          "version": [0, 1],
          "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
          "checksum": "2957dfec6c6f13685809615e45f6c13f11910aece8190b6284c33459cf05d2cc",
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": initMsg,
          "contract_name": "mod_balances"
        }
      }
    }, executeGas)

    const exec1 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload1.codeId,
          "version": [0, 1],
          "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
          "checksum": "be1b58df54ed7ac79bad071e74775a60027764d72ea6707563e3eb65f8fea746",
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": initMsg,
          "contract_name": "mod_dao"
        }
      }
    }, executeGas)

    const exec2 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload2.codeId,
          "version": [0, 1],
          "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
          "checksum": "40dc08420213052973e625bde03b72c18ffd74a02de267acbd986db2481b8648",
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": initMsg,
          "contract_name": "mod_generic"
        }
      }
    }, executeGas)

    const exec3 = await this.client.execute(sender, factoryAddress, {
      "deploy": {
        "kind": "library",
        "module_instantiate_info": {
          "code_id": upload3.codeId ,
          "version": [0, 1],
          "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
          "checksum": "c41d815fbfb4e4db8404a921a468effad61764cb946d3f834bd480bb5eff17a2",
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": initMsg,
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
