SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh
if [ -z "$1" ]; then
  echo "Must provide contract address"
  exit 1
elif [ -z "$2" ]; then
  echo "Must provide task hash"
  exit 1
fi
CONTRACT="$1"
TASK_HASH="$2"

RM='{"remove_task":{"task_hash":"'$TASK_HASH'"}}'
junod tx wasm execute $CONTRACT "$RM" --amount 1000000ujunox --from signer $TXFLAG -y
