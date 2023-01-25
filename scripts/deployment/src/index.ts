import {setupWasmExtension, SigningCosmWasmClient} from '@cosmjs/cosmwasm-stargate'
import {DirectSecp256k1HdWallet} from "@cosmjs/proto-signing"
import {HttpBatchClient, Tendermint34Client, TxResponse} from "@cosmjs/tendermint-rpc"
import {QueryClient} from "@cosmjs/stargate";
import {fromHex} from "@cosmjs/encoding";
import {config} from "dotenv"
import {GasPrice, StdFee} from "@cosmjs/stargate"
import * as fs from "fs"
import * as util from "util"
config({ path: '.env' })
// Get values from the environment variables located in the .env file
const seedPhrase: string = process.env.SEED_PHRASE
const prefix: string = process.env.PREFIX
const endpoint: string = process.env.RPC_ENDPOINT

const artifactsRoot = `${process.cwd()}/../../artifacts`

const start = async () => {
    const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, { prefix })
    const userAddress = (await signerWallet.getAccounts())[0].address
    console.log('userAddress', userAddress)
    const cwClient = await SigningCosmWasmClient.connectWithSigner(endpoint, signerWallet, { prefix, gasPrice: GasPrice.fromString("0.025stake")})

    const factoryWasm = fs.readFileSync(`${artifactsRoot}/croncat_factory.wasm`)
    let uploadFactoryRes = await cwClient.upload(userAddress, factoryWasm, "auto")

    // Ensure transaction succeeded
    const httpBatchClient = new HttpBatchClient(endpoint, {
        batchSizeLimit: 3,
        dispatchInterval: 3000
    })
    const tmClient = await Tendermint34Client.create(httpBatchClient)
    // Keep the line below, as we'll use it later
    const queryClient = QueryClient.withExtensions(tmClient, setupWasmExtension)

    const hash = Buffer.from(fromHex(uploadFactoryRes.transactionHash))
    let txInfo = await tmClient.tx({hash})

    if (txInfo.result.code !== 0) {
        console.error('Transaction failed, got code', txInfo.result.code, hash)
        return
    }

    // Now instantiate the factory
    const factoryId = uploadFactoryRes.codeId
    // We pass it empty '{}' parameters meaning it will make the owner the sender
    const factoryInst = await cwClient.instantiate(userAddress, factoryId, {}, 'CronCat-factory-alpha', 600_000)

    const factoryAddress = factoryInst.contractAddress

    // Manager contract

    // deploy manager contract (from our sender)
    const managerWasm = fs.readFileSync(`${artifactsRoot}/croncat_manager.wasm`)
    const uploadManagerRes = await cwClient.upload(userAddress, managerWasm, "auto")
    const managerId = uploadManagerRes.codeId

    let base64ManagerInst = Buffer.from(JSON.stringify({
        "denom": "ujunox",
        "croncat_factory_addr": factoryAddress,
        "croncat_tasks_key": [
            "t",
            [
                0,
                1
            ]
        ],
        "croncat_agents_key": [
            "a",
            [
                0,
                1
            ]
        ]
    })).toString('base64')

    // instantiate manager contract (from the factory)
    const managerDeployMsg = {
        "deploy": {
            "kind": "manager",
            "module_instantiate_info": {
                "code_id": managerId,
                "version": [
                    0,
                    1
                ],
                "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
                "checksum": "abc123",
                "changelog_url": "https://example.com/lucky",
                "schema": "https://croncat-schema.example.com/version-0-1",
                "msg": base64ManagerInst,
                "contract_name": "croncat-manager--version-0-1"
            }
        }
    }

    const instManagerRes = await cwClient.execute(userAddress, factoryAddress, managerDeployMsg, "auto")
    // console.log('instManagerRes', instManagerRes)
    // console.log('instManagerRes logs', util.inspect(instManagerRes.logs, false, null, true))

    // Boy do I hate indexing like this, folks
    let managerAddress: string = instManagerRes.logs[0].events[1].attributes[0].value

    // Agent contract

    // deploy agent contract (from our sender)
    const agentWasm = fs.readFileSync(`${artifactsRoot}/croncat_agents.wasm`)
    const uploadAgentRes = await cwClient.upload(userAddress, agentWasm, "auto")
    const agentId = uploadAgentRes.codeId

    let base64AgentInst = Buffer.from(JSON.stringify({
        manager_addr: managerAddress
    })).toString('base64')

    // instantiate manager contract (from the factory)
    const agentDeployMsg = {
        "deploy": {
            "kind": "agents",
            "module_instantiate_info": {
                "code_id": agentId,
                "version": [
                    0,
                    1
                ],
                "commit_id": "8e08b808465c42235f961423fcf9e4792ce02462",
                "checksum": "abc123",
                "changelog_url": "https://example.com/lucky",
                "schema": "https://croncat-schema.example.com/version-0-1",
                "msg": Buffer.from(JSON.stringify({
                    manager_addr: "todo: leftoff"
                })),
                "contract_name": "croncat-manager--version-0-1"
            }
        }
    }

    const instAgentRes = await cwClient.execute(userAddress, factoryAddress, managerDeployMsg, "auto")
    const agentAddress: string = instAgentRes.logs[0].events[1].attributes[0].value

    // Show all
    console.info('------ ------ ------')
    console.info(`factory\t code ID ${factoryId},\t address ${factoryAddress}`)
    console.info(`manager\t code ID ${managerId},\t address ${managerAddress}`)
    console.info(`agent\t code ID ${agentId},\t address ${agentAddress}`)

    process.exit()
}

// Start deployment
(() => start())()
