import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { config } from "dotenv"
config({ path: '.env' })
const prefix: string = process.env.PREFIX

export class TaskClient {
  client: SigningCosmWasmClient;

  constructor(client: SigningCosmWasmClient) {
    this.client = client;
  }

  async deploy(artifactsRoot: string, sender: string, factoryAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
    const wasm = fs.readFileSync(`${artifactsRoot}/croncat_tasks.wasm`)
    const uploadRes = await this.client.upload(sender, wasm, uploadGas)
    const codeId = uploadRes.codeId

    // instantiate manager contract (from the factory)
    const deployMsg = {
      "deploy": {
        "kind": "tasks",
        "module_instantiate_info": {
          "code_id": codeId,
          "version": [0, 1],
          "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
          "checksum": "665267d0076b69a971b24eeddce447dbc7e5b280c8997ca52f493d4ed569c284",
          "changelog_url": "https://github.com/croncats",
          "schema": "",
          "msg": Buffer.from(JSON.stringify({
            chain_name: prefix || 'juno',
            croncat_manager_key: ['manager', [0, 0]],
            croncat_agents_key: ['agents', [0, 0]],
            // owner_addr: '',
            // croncat_manager_key: '',
            // croncat_agents_key: '',
            // slot_granularity_time: '',
            // gas_base_fee: '',
            // gas_action_fee: '',
            // gas_query_fee: '',
            // gas_limit: '',
          })).toString('base64'),
          "contract_name": "tasks"
        }
      }
    }
    const instRes = await this.client.execute(sender, factoryAddress, deployMsg, executeGas);
    const address: string = instRes.logs[0].events[1].attributes[0].value

    return [codeId, address];
  }
}
