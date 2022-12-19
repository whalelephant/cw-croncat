#!/bin/sh
DIR=$(pwd)
. "$DIR/scripts/local/start-in-docker.sh"

echo "${Cyan}Transfering some CW-20 to Croncat..." "${NoColor}"

CW20_SEND='{"send": {"contract": "'$CONTRACT_ADDRESS'", "amount": "5", "msg": ""}}'
$BINARY tx wasm execute $CW20_ADDR "$CW20_SEND" --from validator $TXFLAG -y

echo "${Cyan}Creating task with rule..." "${NoColor}"

BASE64_TRANSFER=$(echo -b '{"transfer":{"recipient":"'$AGENT_ADDR'","amount":"5"}}' | base64)
RULES='{
    "create_task": {
        "task": {
            "interval": "Once",
            "boundary": null,
            "stop_on_fail": false,
            "actions": [
                {
                    "msg": {
                        "wasm": {
                            "execute": {
                                "contract_addr": "'$CW20_ADDR'",
                                "msg": "'$BASE64_TRANSFER'",
                                "funds": []
                            }
                        }
                    },
                    "gas_limit": null
                }
            ],
            "rules": [
                {
                    "has_balance_gte": {
                        "address": "'$BOB_ADDR'",
                        "required_balance": {
                            "cw20": {
                                "address": "'$CW20_ADDR'",
                                "amount": "5"
                            }
                        }
                    }
                }
            ],
            "cw20_coins": [
                {
                    "address": "'$CW20_ADDR'",
                    "amount": "5"
                }
            ]
        }
    }
}'
$BINARY tx wasm execute $CONTRACT_ADDRESS "$RULES" --amount "1700004$STAKE" --from validator $TXFLAG -y
echo "${Cyan}Creating task with rule done!" "${NoColor}"

echo "${Cyan}Transfer 5 memecoins rule to pass" "${NoColor}"

CW20_TRANSFER='{"transfer": {"recipient": "'$BOB_ADDR'", "amount": "5"}}'
$BINARY tx wasm execute $CW20_ADDR "$CW20_TRANSFER" --from validator $TXFLAG -y
echo "${Cyan}Transfer 5 memecoins rule to pass done!" "${NoColor}"
