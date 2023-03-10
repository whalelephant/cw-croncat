import { ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { Coin, StdFee, QueryClient, calculateFee } from "@cosmjs/stargate";
import * as fs from "fs"
import { getGitHash, getChecksums, getContractVersionFromCargoToml } from './utils'
import { DeploySigner } from "./signer"

export class AgentClient {
    client: DeploySigner;
    querier: any;
    uploadGas: any;
    executeGas: any;
    codeId: number;
    address: string;

    constructor(client: DeploySigner, address?: string, querier?: QueryClient) {
        this.client = client;
        this.querier = querier || client.querier;

        if (address) this.address = address;
    }

    async deploy(
        artifactsRoot: string,
        factoryAddress: string,
        agentsAddresses: string[],
    ): Promise<[number, string]> {
        if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
        this.uploadGas = calculateFee(4_400_000, this.client.defaultGasPrice)
        this.executeGas = calculateFee(555_000, this.client.defaultGasPrice)
		const wasm = fs.readFileSync(`${artifactsRoot}/croncat_agents.wasm`)
        const uploadRes = await this.client.client.upload(this.client.accounts.deployer, wasm, this.uploadGas)
        this.codeId = uploadRes.codeId

        const githash = await getGitHash()
        const checksums = await getChecksums()

        // get the version from cargo
        const version = await getContractVersionFromCargoToml('croncat-agents')

        // instantiate manager contract (from the factory)
        const deployMsg = {
            "deploy": {
                "kind": "agents",
                "module_instantiate_info": {
                    "code_id": this.codeId,
                    "version": version,
                    "commit_id": githash || '-',
                    "checksum": checksums.agents || '-',
                    "changelog_url": "https://github.com/croncats",
                    "schema": "",
                    "msg": Buffer.from(JSON.stringify({
                        "pause_admin": `${this.client.accounts.pause_admin}`,
                        "version": `${version[0]}.${version[1]}`,
                        "public_registration": false,
                        "allowed_agents": agentsAddresses,
                        "croncat_manager_key": ["manager", version || [0, 1]],
                        "croncat_tasks_key": ["tasks", version || [0, 1]],
                        // agent_nomination_duration: '',
                        // min_tasks_per_agent: '',
                        // min_coins_for_agent_registration: '',
                        // agents_eject_threshold: '',
                        // min_active_agent_count: '',
                    })).toString('base64'),
                    "contract_name": "agents"
                }
            }
        }
        const instRes = await this.client.client.execute(this.client.accounts.deployer, factoryAddress, deployMsg, this.executeGas);
        this.address = instRes.logs[0].events[1].attributes[0].value

        return [this.codeId, this.address];
    }

    async getAgents(): Promise<any> {
        if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
        const q = { get_agent_ids: {} };
        const response = await this.querier.wasm.queryContractSmart(this.address, q);
        return response;
    }

    async status(sender: string): Promise<any> {
        if (!this.querier) return Promise.reject(`No querier found for ${this.client.chain.chain_name}!`)
        const q = { get_agent: { account_id: sender } };
        const response = await this.querier.wasm.queryContractSmart(this.address, q);
        return response;
    }

    async register(sender: string, gas: StdFee): Promise<ExecuteResult> {
        if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
        const msg = { register_agent: { payable_account_id: sender } };
        const response = await this.client.client.execute(sender, this.address, msg, gas);
        return response;
    }

    async update(sender: string, gas: StdFee): Promise<ExecuteResult> {
        if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
        const msg = { update_agent: { payable_account_id: sender } };
        const response = await this.client.client.execute(sender, this.address, msg, gas);
        return response;
    }

    async unregister(sender: string, gas: StdFee): Promise<ExecuteResult> {
        if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
        const msg = { unregister_agent: {} };
        const response = await this.client.client.execute(sender, this.address, msg, gas);
        return response;
    }

    async checkIn(sender: string, gas: StdFee): Promise<ExecuteResult> {
        if (!this.client.client) return Promise.reject(`No signer found for ${this.client.chain.chain_name}!`)
        const msg = { check_in_agent: {} };
        const response = await this.client.client.execute(sender, this.address, msg, gas);
        return response;
    }
}
