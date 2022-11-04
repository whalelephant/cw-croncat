## Set registry contract address

```bash
REGISTRY_CONTRACT_ADDRESS=juno1yfl86vq2qy9evvu5dc6w53tv3dwffm4xxnhdymzzj52e0757jc3q5g06eg
```
## Query registrations
```bash
junod query wasm contract-state smart $REGISTRY_CONTRACT_ADDRESS '{"get_registration":{"contract_name": "cw-code-id-registry", "chain_id": "uni-5"}}' --node "https://rpc.uni.junonetwork.io:443"
```
## Query registrations

```bash
TXFLAG="--chain-id uni-5 --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.3 --broadcast-mode block"
REGISTER_MSG='{"register":{"contract_name": "cw-code-id-registry", "version": "0.1.0", "chain_id": "uni-5", "code_id": 1746, "checksum": "8608F8126D64B39C10433CB09481BA09299C208FF1A5E5B3DEAF9F1DEC6B2F2A"}}'

junod tx wasm execute $REGISTRY_CONTRACT_ADDRESS "$REGISTER_MSG" --from $YOUR_KEY_NAME --node "https://rpc.uni.junonetwork.io:443" $TXFLAG
```