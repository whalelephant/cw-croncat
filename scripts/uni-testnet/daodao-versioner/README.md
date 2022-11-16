## Set registry contract address

```bash
source ./scripts/uni-testnet/daodao-versioner/.env
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
VERSIONER_ADDRESS=juno1ydnsddskwek69hm8gkem5h6k48hnsuqqyv346y34xca0afnnznsqdd356j
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
