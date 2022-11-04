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

$SCRIPTS_PATH/build.sh
echo "Initializing vars"
. $SH_DIR/base/init-vars.sh

VERSIONER_ADDRESS="juno1shmnkdkgzz3le4g0r6h475jfe2llsdgwutcz8vrnqcmk8th4hfeqa4nw03"
DAODAO_ADDR="juno16jy8py9c2jsu08rwjl534exss7nwp6p78et73wuhh5lxhrddvl8q4vz55q"

CREATE_MSG='{"create_contract_versioner":{"daodao_addr": "$DAODAO_ADDR","name": "cw-code-id-registry", "chain_id": "uni-5"}}'
junod tx wasm execute $VERSIONER_ADDRESS "$CREATE_MSG" --amount 1000000ujunox --from juno183ct2qqalrkch350zyqwesut7mc976ypj3k6yt $TXFLAG -y
