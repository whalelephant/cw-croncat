import yargs from 'yargs'
import { hideBin } from 'yargs/helpers'
import { config } from "dotenv"
config({ path: '.env' })
import * as fs from "fs"
import { getChainByChainName, getSupportedNetworks } from './utils'
import { DeploySigner } from "./signer"
import { FactoryClient } from './factory';
import { ManagerClient } from './manager';
import { TaskClient } from './tasks';
import { AgentClient } from './agents';
import { ModulesClient } from './modules';

const artifactsRoot = process.env.WASM_BUILD_FOLDER ? `../../${process.env.WASM_BUILD_FOLDER}` : `${process.cwd()}/../../artifacts`

const deployNetwork = async (cwClient) => {
    console.info(`🤖 Starting ${cwClient.chain.pretty_name} Deployment 🤖`)

    // Check for balances first, and attempt to dust other accounts if needed
    const accountBalances = await cwClient.getAllAccountsBalances()
    const sendAmount = 300_000 // `EX: 0.3 juno` - enough for an agent to run 4 tasks and withdraw
    const deployer = accountBalances.find(ab => ab.id === 'deployer')
    // first make sure we can even do this thing
    if (!deployer || !deployer.amount || parseInt(`${deployer.amount}`, 10) < accountBalances.length * sendAmount) {
        console.error(`${deployer.id} ${deployer.address} does not have enough funds! Needs at least ${accountBalances.length * sendAmount} ${cwClient.fee_token.denom}`)
        process.exit()
    }

    const ps = []
    accountBalances.forEach(ab => {
        const curAmt = parseInt(`${ab.amount}`, 10)
        if (ab.id != 'deployer' && curAmt < sendAmount) {
            ps.push([ab.id, sendAmount - curAmt])
        }
    })
    // we gotsta send funds for all to participate bruh
    // Needs to be sequential, or we get: "account sequence mismatch"
    // NOTE: Someday we set this up in single batch TX 💸
    if (ps.length) {
        const series = async (ids) => {
            for (const act of ids) {
                console.log(`Sending ${act[1]} ${cwClient.fee_token.denom} to ${act[0]}`)
                await cwClient.sendTokens(act[0], act[1])
            }
        }

        await series(ps)
        await cwClient.listAccounts()
    }

    // Factory
    const factoryClient = new FactoryClient(cwClient);
    const [factoryId, factoryAddress] = await factoryClient.deploy(artifactsRoot);
    console.info(`🏭 Factory Done`, factoryId, factoryAddress)

    // Manager
    const managerClient = new ManagerClient(cwClient);
    const [managerId, managerAddress] = await managerClient.deploy(artifactsRoot, factoryAddress);
    console.info(`🏗️  Manager Done`, managerId, managerAddress)

    // Tasks
    const taskClient = new TaskClient(cwClient);
    const [taskContractCodeId, taskContractAddr] = await taskClient.deploy(artifactsRoot, factoryAddress);
    console.info(`🏗️  Tasks Done`, taskContractCodeId, taskContractAddr)

    // Agents
    const agentClient = new AgentClient(cwClient);
    const [agentContractCodeId, agentContractAddr] = await agentClient.deploy(
        artifactsRoot,
        factoryAddress,
        // NOTE: Agent 1-5 exist
        [cwClient.accounts.agent1, cwClient.accounts.agent2],
    );
    console.info(`🏗️  Agents Done`, agentContractCodeId, agentContractAddr)

    // Modules
    const modulesClient = new ModulesClient(cwClient);
    const modules = await modulesClient.deploy(artifactsRoot, factoryAddress);
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
    await fs.writeFileSync(`${artifactsRoot}/${cwClient.chain.chain_name}-deployed_contracts.json`, JSON.stringify(output))

    // return the factory address for final file writer
    return {
        chain_name: cwClient.chain.chain_name,
        code_id: factoryId,
        address: factoryAddress
    }
}

// Bootstrap all networks before deploying contexts
const start = async () => {
    const args = yargs(hideBin(process.argv)).argv
    let chainName
    let networks = []
    if (args._ && args._.length > 0) {
        chainName = args._[0]
        const chain = getChainByChainName(chainName)
        if (chain) {
            networks = [chain]
        } else {
            console.error(`Couldn't find ${chainName}, please try different chain_name and try again.`)
            process.exit()
        }
    } else {
        networks = getSupportedNetworks()
    }
    if (!networks || !networks.length) process.exit();
    // Instantiate all the clients needed
    const networkClients = {}
    try {
        await Promise.all(networks.map(async n => {
            const ds = new DeploySigner()
            networkClients[n.chain_name] = await ds.init(n)
            return n
        }))
    } catch (e) {
        console.log(e);
    }

    if (!Object.keys(networkClients) || !Object.keys(networkClients).length) process.exit()

    // loop all clients and display their address/balances
    const p = []
    Object.keys(networkClients).forEach(k => p.push(deployNetwork(networkClients[k])))
    const factoryDeploys = await Promise.all(p)

    // Store this output, for use in agent & website envs
    // TODO: Setup a read, so we don't overwrite other networks not in this list!!
    if (factoryDeploys && factoryDeploys.length) {
        await fs.writeFileSync(`${artifactsRoot}/deployed_factories.json`, JSON.stringify(factoryDeploys))
        console.table(factoryDeploys)
    }

    process.exit()
}

// Start deployment
(() => start())()



