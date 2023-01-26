import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"

export class AgentClient {
    client: SigningCosmWasmClient;

    constructor(client: SigningCosmWasmClient) {
        this.client = client;
    }

    async deploy(artifactsRoot: string, sender: string, factoryAddress: string, managerAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
        const wasm = fs.readFileSync(`${artifactsRoot}/croncat_agents.wasm`)
        const uploadRes = await this.client.upload(sender, wasm, uploadGas)
        const codeId = uploadRes.codeId

        // instantiate manager contract (from the factory)
        const deployMsg = {
            "deploy": {
                "kind": "agents",
                "module_instantiate_info": {
                    "code_id": codeId,
                    "version": [0, 1],
                    "commit_id": "6ffbf4aa3617f978a07b594adf8013f19a936331",
                    "checksum": "8f19d75a7523605190654125e476c0bc14d1eb7ffa7524aa280221f52a244ccf",
                    "changelog_url": "https://github.com/croncats",
                    "schema": "",
                    "msg": Buffer.from(JSON.stringify({
                        manager_addr: managerAddress
                    })).toString('base64'),
                    "contract_name": "agents"
                }
            }
        }
        const instAgentRes = await this.client.execute(sender, factoryAddress, deployMsg, executeGas);
        const address: string = instAgentRes.logs[0].events[1].attributes[0].value

        return [codeId, address];
    }

    async registerAgent(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { register_agent: { payable_account_id: sender } };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }

    async unregisterAgent(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { unregister_agent: {} };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }
}
