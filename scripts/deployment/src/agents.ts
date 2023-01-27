import { ExecuteResult, SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { QueryClient } from "@cosmjs/stargate";
import { StdFee } from "@cosmjs/stargate";
import { getGitHash, getChecksums } from './utils'
import * as fs from "fs"

export class AgentClient {
    client: SigningCosmWasmClient;
    querier: any;

    constructor(client: SigningCosmWasmClient, querier?: QueryClient) {
        this.client = client;
        this.querier = querier;
    }

    async deploy(artifactsRoot: string, sender: string, factoryAddress: string, managerAddress: string, tasksAddress: string, uploadGas: StdFee, executeGas: StdFee): Promise<[number, string]> {
        const wasm = fs.readFileSync(`${artifactsRoot}/croncat_agents.wasm`)
        const uploadRes = await this.client.upload(sender, wasm, uploadGas)
        const codeId = uploadRes.codeId
        const githash = await getGitHash()
        const checksums = await getChecksums()

        // instantiate manager contract (from the factory)
        const deployMsg = {
            "deploy": {
                "kind": "agents",
                "module_instantiate_info": {
                    "code_id": codeId,
                    "version": [0, 1],
                    "commit_id": githash,
                    "checksum": checksums.agents,
                    "changelog_url": "https://github.com/croncats",
                    "schema": "",
                    "msg": Buffer.from(JSON.stringify({
                        manager_addr: managerAddress,
                        tasks_addr: tasksAddress
                    })).toString('base64'),
                    "contract_name": "agents"
                }
            }
        }
        const instAgentRes = await this.client.execute(sender, factoryAddress, deployMsg, executeGas);
        const address: string = instAgentRes.logs[0].events[1].attributes[0].value

        return [codeId, address];
    }

    async status(sender: string, contractAddr: string): Promise<any> {
        const q = { get_agent: { account_id: sender } };
        const response = await this.querier.wasm.queryContractSmart(contractAddr, q);
        return response;
    }

    async register(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { register_agent: { payable_account_id: sender } };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }

    async update(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { update_agent: { payable_account_id: sender } };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }

    async unregister(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { unregister_agent: {} };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }

    async checkIn(sender: string, contractAddr: string, gas: StdFee): Promise<ExecuteResult> {
        const msg = { check_in_agent: {} };
        const response = await this.client.execute(sender, contractAddr, msg, gas);
        return response;
    }
}
