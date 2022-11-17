## Set registry contract address

```bash
TXFLAG="--chain-id uni-5 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block --node https://rpc.uni.junonetwork.io:443"
NODE="--node https://rpc.uni.junonetwork.io:443"
REGISTRY_CONTRACT_ADDRESS=juno1k2z6m5duj8hnyc7wfk43wzxexc65zg0kp4pv2ccf83y4fe533c3qynes6j
CRONCAT_ADDRESS=juno1ns5utq5s4np90fjtsfzl9zzlzpppdcntjg4y8e4quejha373zfcq94mqtw
DAODAO_ADDR=juno1jx33vf2w36uqa3e0qq68azs2ar2sr05vgqswsxsukasa39jpxs6qqdue8z
SIGNER_ADDR=$(junod keys show signer --address)
```

## Query registrations
```bash
junod query wasm contract-state smart $REGISTRY_CONTRACT_ADDRESS '{"get_registration":{"name": "cw-code-id-registry", "chain_id": "uni-5"}}' --node "https://rpc.uni.junonetwork.io:443"
```
## Register new version in registrar

```bash
REGISTER_MSG='{"register":{"contract_name": "cw-code-id-registry", "version": "0.1.0", "chain_id": "uni-5", "code_id": 1746, "checksum": "8608F8126D64B39C10433CB09481BA09299C208FF1A5E5B3DEAF9F1DEC6B2F2A"}}'
junod tx wasm execute $REGISTRY_CONTRACT_ADDRESS "$REGISTER_MSG" --from signer --node "https://rpc.uni.junonetwork.io:443" $TXFLAG
```
## Deploy versioner
```bash
./scripts/uni-testnet/versioner-deploy.sh -w -c
```

## Create versioner new entry

```bash
VERSIONER_ADDRESS=juno1tkw3hrprd75vm5a73pd57zg7qhadv6qd02lg6u7z7fht48ldwgmqtw0f6m
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
