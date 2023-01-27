import { setupWasmExtension, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { HttpBatchClient, Tendermint34Client, TxResponse } from "@cosmjs/tendermint-rpc"
import { QueryClient } from "@cosmjs/stargate";
import { fromHex } from "@cosmjs/encoding";
import { config } from "dotenv"
import { GasPrice, StdFee, calculateFee } from "@cosmjs/stargate"
import * as fs from "fs"
import * as util from "util"
import { FactoryClient } from './factory';
import { ManagerClient } from './manager';
import { TaskClient } from './tasks';
import { AgentClient } from './agents';
import { ModulesClient } from './modules';
config({ path: '.env' })
// Get values from the environment variables located in the .env file
const seedPhrase: string = process.env.SEED_PHRASE
const prefix: string = process.env.PREFIX
const endpoint: string = process.env.RPC_ENDPOINT
const denom: string = process.env.DENOM
const defaultGasPrice = GasPrice.fromString(`0.025${denom}`)
const artifactsRoot = `${process.cwd()}/../../artifacts`

// Gas vals
const uploadGas = calculateFee(4_000_000, defaultGasPrice)
const executeGas = calculateFee(555_000, defaultGasPrice)

const prettified_out=(o:object)=>{
    console.info(JSON.stringify(o, null, '\t'));
}
const start = async () => {
    console.info(`🤖 Starting Deployment 🤖`)

    const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, { prefix })
    const userAddress = (await signerWallet.getAccounts())[0].address
    const cwClient = await SigningCosmWasmClient.connectWithSigner(endpoint, signerWallet, { prefix, gasPrice: defaultGasPrice })

    // Ensure transaction succeeded
    const httpBatchClient = new HttpBatchClient(endpoint, {
        batchSizeLimit: 2,
        dispatchInterval: 500
    })
    const tmClient = await Tendermint34Client.create(httpBatchClient)
    // Keep the line below, as we'll use it later
    // const queryClient = QueryClient.withExtensions(tmClient, setupWasmExtension)

    // Factory
    var factoryClient = new FactoryClient(cwClient);
    var [factoryId, factoryAddress] = await factoryClient.deploy(artifactsRoot, userAddress, uploadGas, executeGas);
    console.info(`🏭 Factory Done`)

    // Manager
    var managerClient = new ManagerClient(cwClient);
    var [managerId, managerAddress] = await managerClient.deploy(artifactsRoot, userAddress, factoryAddress, uploadGas, executeGas);
    console.info(`🏗️  Manager Done`)

    // Tasks
    var taskClient = new TaskClient(cwClient);
    var [taskContractCodeId, taskContractAddr] = await taskClient.deploy(artifactsRoot, userAddress, factoryAddress, uploadGas, executeGas);
    console.info(`🏗️  Tasks Done`)

    // Agents
    var agentClient = new AgentClient(cwClient);
    var [agentContractCodeId, agentContractAddr] = await agentClient.deploy(artifactsRoot, userAddress, factoryAddress, managerAddress, taskContractAddr, uploadGas, executeGas);
    console.info(`🏗️  Agents Done`)

    // Modules
    var modulesClient = new ModulesClient(cwClient);
    var modules = await modulesClient.deploy(artifactsRoot, userAddress, factoryAddress, uploadGas, executeGas);
    console.info(`🏗️  Modules Done`)

    // Show all
    const output = [
        { name: 'factory', code_id: factoryId, address: factoryAddress },
        { name: 'manager', code_id: managerId, address: managerAddress },
        { name: 'agent', code_id: agentContractCodeId, address: agentContractAddr },
        { name: 'tasks', code_id: taskContractCodeId, address: taskContractAddr },
        ...modules,
    ]
    console.table(output)

    // Store this output, for use in agent & website envs
    await fs.writeFileSync(`${artifactsRoot}/${process.env.CHAIN_ID}_deployed_contracts.json`, JSON.stringify(output))

    process.exit()
}

// Start deployment
(() => start())()



