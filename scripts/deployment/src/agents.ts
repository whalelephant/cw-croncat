import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import * as fs from "fs"

export class AgentClient {
    client: SigningCosmWasmClient;

    constructor(client: SigningCosmWasmClient) {
        this.client = client;
    }

    async registerAgent(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { register_agent: {payable_account_id:sender} };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }
    async deploy(artifactsRoot: string, sender: string, factoryAddress: string, managerAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
        const agentWasm = fs.readFileSync(`${artifactsRoot}/croncat_agents.wasm`)
        const uploadAgentRes = await this.client.upload(sender, agentWasm, uploadGas)
        const agentContractCodeId = uploadAgentRes.codeId

        // instantiate manager contract (from the factory)
        const agentDeployMsg = {
            "deploy": {
                "kind": "agents",
                "module_instantiate_info": {
                    "code_id": agentContractCodeId,
                    "version": [
                        0,
                        1
                    ],
                    "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
                    "checksum": "abc123",
                    "changelog_url": "https://example.com/lucky",
                    "schema": "https://croncat-schema.example.com/version-0-1",
                    "msg": Buffer.from(JSON.stringify({
                        manager_addr: managerAddress
                    })).toString('base64'),
                    "contract_name": "croncat-agents--version-0-1"
                }
            }
        }
        const instAgentRes = await this.client.execute(sender, factoryAddress, agentDeployMsg, executeGas);
        // console.log('instAgentRes logs', util.inspect(instAgentRes.logs, false, null, true))
        const agentAddress: string = instAgentRes.logs[0].events[1].attributes[0].value

        return [agentContractCodeId, agentAddress];
    }


}
