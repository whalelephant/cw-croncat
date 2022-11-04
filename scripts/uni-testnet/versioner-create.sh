#!/bin/sh
set -e
source ~/.profile
SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
SC_PATH="$(cd -P "$(dirname "${SH_PATH}")/../..";pwd)"
SCRIPTS_PATH="$(cd -P "$(dirname "${SH_PATH}")/..";pwd)"

echo "CONTRACT-DIR: $SC_PATH"
echo "SCRIPT-DIR: $SH_DIR"
cd $SC_PATH

echo "Initializing vars"
. $SH_DIR/base/init-vars.sh

VERSIONER_ADDRESS="juno1nhvl8vk6hwxkc2yx0k2552z4842c72fmtl0vfwwl7w27q0rr27vqh3zykw"
DAODAO_ADDR="juno16jy8py9c2jsu08rwjl534exss7nwp6p78et73wuhh5lxhrddvl8q4vz55q"

echo $DAODAO_ADDR
echo $VERSIONER_ADDRESS
CREATE_MSG='{
	"create_versioner": {
		"daodao_addr": "'$DAODAO_ADDR'",
		"name": "cw-code-id-registry",
		"chain_id": "uni-5"
	}
}'
#CREATE_MSG='{"query_result":{}}'
echo $CREATE_MSG
junod tx wasm execute $VERSIONER_ADDRESS "$CREATE_MSG" --amount 1000000ujunox --from juno183ct2qqalrkch350zyqwesut7mc976ypj3k6yt $TXFLAG -y
