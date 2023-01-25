

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/dec.sh
ALICE_BALANCE=$($BINARY q bank balances $($BINARY keys show alice --address)  $NODE)
echo "${Green}Alice Balance :" $ALICE_BALANCE "${NoColor}"
BOB_BALANCE=$($BINARY q bank balances $($BINARY keys show bob --address) $NODE)
echo "${Green}Bob Balance :" $BOB_BALANCE "${NoColor}"
OWNER_BALANCE=$($BINARY q bank balances $($BINARY keys show owner --address) $NODE)
echo "${Green}Owner Balance :" $OWNER_BALANCE "${NoColor}"
AGENT_BALANCE=$($BINARY q bank balances $($BINARY keys show agent --address) $NODE)
echo "${Green}Agent Balance :" $AGENT_BALANCE "${NoColor}"
USER_BALANCE=$($BINARY q bank balances $($BINARY keys show user --address) $NODE)
echo "${Green}User Balance :" $USER_BALANCE "${NoColor}"
SIGNER_BALANCE=$($BINARY q bank balances $($BINARY keys show signer --address) $NODE)
echo "${Green}Signer Balance :" $SIGNER_BALANCE "${NoColor}"