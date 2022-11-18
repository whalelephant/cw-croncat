## Set registry contract address

```bash
NODE="--node https://rpc.uni.junonetwork.io:443"
TXFLAG="--node https://juno-testnet-rpc.polkachu.com:443 --chain-id uni-5 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"
REGISTRY_CONTRACT_ADDRESS=juno1k2z6m5duj8hnyc7wfk43wzxexc65zg0kp4pv2ccf83y4fe533c3qynes6j
CRONCAT_ADDRESS=juno1vknyvkygchgjc8wlqy4wrw0adlm3z7csuga40vmmrsz73fra6sqqhewxc0
DAODAO_ADDR=juno17wtmkacj5as53dsp6he6u9cqxjgv706eqmv0yt7p9y7ejwdpnu5s7np2fr
SIGNER_ADDR=$(junod keys show signer --address)

export REGISTRY_CONTRACT_ADDRESS
export CRONCAT_ADDRESS
export DAODAO_ADDR
```

## Query registrations
```bash
junod query wasm contract-state smart $REGISTRY_CONTRACT_ADDRESS '{"get_registration":{"name": "cw-code-id-registry", "chain_id": "uni-5"}}' --node "https://rpc.uni.junonetwork.io:443"
```
## Register new version in registrar

```bash
REGISTER_MSG='{"register":{"contract_name": "cw-code-id-registry", "version": "0.1.1", "chain_id": "uni-5", "code_id": 1749, "checksum": "8608F8126D64B39C10433CB09481BA09299C208FF1A5E5B3DEAF9F1DEC6B2F2A"}}'
junod tx wasm execute $REGISTRY_CONTRACT_ADDRESS "$REGISTER_MSG" --from signer --node https://juno-testnet-rpc.polkachu.com:443 --chain-id uni-5 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block -y
```
## Deploy versioner
```bash
./scripts/uni-testnet/versioner-deploy.sh -w -c
```

## Create versioner new entry

```bash
VERSIONER_ADDRESS=juno1ra56r3e4dwc9gyum5jufkd0n87zqvn3hpk2cxm46h4wgp6xnw3tsjxumzx
./scripts/uni-testnet/versioner-create.sh $VERSIONER_ADDRESS $DAODAO_ADDR
```
## Get croncat tasks

```bash
./scripts/uni-testnet/get-tasks.sh $CRONCAT_ADDRESS
```
## Call for task execution on croncat

```bash
./scripts/uni-testnet/register-agent-then-proxy-call.sh $CRONCAT_ADDRESS $SIGNER_ADDR
```

## Remove versioner if needed

```bash
./scripts/uni-testnet/versioner-remove.sh $VERSIONER_ADDRESS
```
