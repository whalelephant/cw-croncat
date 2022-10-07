# Examples

## Local setup

**Warning**: running this script will remove any existing agent key pairs you have stored in `~/.croncatd/agents.json` as well as the `~./juno` which is used by your local `junod`.

You will need `junod` which you can get installed by following these directions:
https://docs.junonetwork.io/validators/getting-setup

After following directions, you should be able to run:

    junod version

and see: `v10.0.0`

Ensure you have a relatively clean set of keys for your local Juno chain. You can see a list of keys by running:

    junod keys list

and can delete them with:

    junod keys delete <name>

Run this script to remove old agent and Juno chain info, create new keys, deploy and instantiate the Croncat Manager contract, and add a simple payroll task that pays Alice and Bob a little bit of `stake` tokens every 3 blocks:

    ./start.sh

A `junod` process is running in the background now, and we can start our agent.

Clone and run the Croncat Agent:

```
git clone https://github.com/CronCats/croncat-rs.git
cd croncat-rs
git checkout seed-phrase-flag
cargo run generate-mnemonic --mnemonic="shove click bless section used eye able chaos welcome peasant base apart issue reduce sphere oven salmon glow distance strategy tortoise spot grunt area"
cargo run register-agent
cargo run go
```

In a separate tab you can see the balances by running:

    ./balances.sh

If you run the previous scripted repeatedly you'll see the simple, automated payroll is operational.

To stop the local Juno chain, you may run:

    ./stop.sh

## Misc

### Create a recurring task to send 1 testnet Juno to two addresses (TODO, really recurring?)
    ./testnet_create_recurring_task.sh juno123contractaddress alice

### Create staking task
    ./testnet_create_staking_task.sh juno123contractaddress alice