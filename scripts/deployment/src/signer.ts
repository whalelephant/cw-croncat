import type { Chain } from "@chain-registry/types"
import { coins, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { stringToPath } from "@cosmjs/crypto"
import { HttpBatchClient, Tendermint34Client } from "@cosmjs/tendermint-rpc"
import { SigningCosmWasmClient, setupWasmExtension } from "@cosmjs/cosmwasm-stargate";
import axios from "axios"
import { QueryClient } from "@cosmjs/stargate";
import { GasPrice } from "@cosmjs/stargate";
import { config } from "dotenv"
config({ path: '.env' })
import { pauseAdmins } from './utils'

const seedPhrase: string = process.env.SEED_PHRASE

export class DeploySigner {
	client: SigningCosmWasmClient;
  querier: any;
  chain: Chain;
  prefix: string;
  defaultGasPrice: GasPrice;
  // {
  //   denom: 'ujunox',
  //   low_gas_price: 0.03,
  //   average_gas_price: 0.04,
  //   high_gas_price: 0.05
  // }
  fee_token: any;
  private mnemonic: string;
  accounts: {
    // agent1: juno123abc...
    [key: string]: string;
  }
  // TODO: Add "pause_admin" address, based on .env config
  pauseAdmin: string;

  constructor() {
	}

  // NOTE: Prefix is the Bech32 prefix for given network
	async init(chain: Chain) {
    this.mnemonic = seedPhrase;
    this.chain = chain;
    this.prefix = chain.bech32_prefix;
    this.pauseAdmin = pauseAdmins[chain.chain_name]
    const prefix = this.prefix;
    this.fee_token = chain.fees.fee_tokens[0];
    this.defaultGasPrice = GasPrice.fromString(`${this.fee_token.average_gas_price}${this.fee_token.denom}`)
    const signerWallet = await DirectSecp256k1HdWallet.fromMnemonic(seedPhrase, {
      prefix,
      hdPaths: [
        // for easier coinage management
        stringToPath(`m/44'/118'/0'/0/0`),
        stringToPath(`m/44'/118'/0'/0/1`),
        stringToPath(`m/44'/118'/0'/0/2`),
        stringToPath(`m/44'/118'/0'/0/3`),
        stringToPath(`m/44'/118'/0'/0/4`),
        stringToPath(`m/44'/118'/0'/0/5`),
        stringToPath(`m/44'/118'/0'/0/6`),
      ]
    })
    const accts = await signerWallet.getAccounts()
    this.accounts = {
      deployer: accts[0].address,
      treasury: accts[1].address,
      agent1: accts[2].address,
      agent2: accts[3].address,
      agent3: accts[4].address,
      agent4: accts[5].address,
      agent5: accts[6].address,
      pause_admin: this.pauseAdmin,
    }
    // console.table(this.accounts);

    // Ping ALL rpc providers and go with the one that resolves the fastest, LFG
    let p = []
    chain.apis.rpc.forEach(r => p.push(axios.get(`${r.address}/status`)))

    let endpoint
    try {
      const rpcFin = await Promise.any(p)
      if (rpcFin.status === 200) endpoint = rpcFin.config.url.replace('/status', '')
      // console.log('RPC endpoint won the race:', endpoint);
    } catch (e) {
      return Promise.reject(e)
    }
    if (!endpoint) return

    // signer client
    const options = { prefix: this.prefix, gasPrice: this.defaultGasPrice }

    // osmosis testnet doesnt support clients correctly!?.............
    if (chain.chain_name !== 'osmosistestnet') {
      try {
        this.client = await SigningCosmWasmClient.connectWithSigner(endpoint, signerWallet, options)
      } catch (e) {
        console.log('failed to create client for', prefix, e);
        return Promise.reject(e)
      }

      try {
        const httpBatchClient = new HttpBatchClient(`${endpoint}`, {
          batchSizeLimit: 10,
          dispatchInterval: 500
        })
        const tmClient = await Tendermint34Client.create(httpBatchClient)
        this.querier = QueryClient.withExtensions(tmClient, setupWasmExtension)
      } catch (e) {
        return Promise.reject(e)
      }
    } else {
      console.log('TODO: Fix osmosistestnet!', endpoint, chain);
    }

    return this
	}

  // loop accounts for all networks & get each balance, print to console
  async listAccounts() {
    const balances = await this.getAllAccountsBalances()
    console.info(`💸 ${this.chain.pretty_name} Accounts 💸`)
    console.table(balances)
  }

  // Query all balances
  async getAllAccountsBalances() {
    const p = []
    Object.keys(this.accounts).forEach(a => p.push(this.getBalance(a)))
    const balances = await Promise.all(p)
    return balances
  }

  // For simple address funding
  async getBalance(id) {
    if (!this.client) return;
    try {
      const res = await this.client.getBalance(this.accounts[id], this.fee_token.denom)
      return {
        id,
        address: this.accounts[id],
        // For maximum re-use
        ...res,
        // balance: `${res.amount} ${res.denom}`,
        note: (res.amount === '0' || !res.amount) && id === 'deployer' ? `Needs funds! All other accounts rely on this one!` : ''
      }
    } catch (e) {
      console.log(`No funds found for ${id}`, e)
    }
  }

  // For simple address funding
  async sendTokens(id, amount) {
    if (!this.client) return;
    try {
      await this.client.sendTokens(this.accounts.deployer, this.accounts[id], coins(amount || 5_000_000, this.fee_token.denom), "auto", ``)
    } catch (e) {
      console.log(`Fund ${id} ERROR`, e)
    }
  }
}