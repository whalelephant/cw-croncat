source ~/.profile

SH_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
SH_DIR="$(cd -P "$(dirname "${SH_PATH}")";pwd)"
. $SH_DIR/base/init-vars.sh
# if [ -z "$1" ]; then
#   echo "Must provide contract address"
#   exit 1
# elif [ -z "$2" ]; then
#   echo "Must provide user address"
#   exit 1
# fi

CONTRACT="juno15ssquhz95cqjh9774cahtv0zud8sffwh86cfxp5a08eatg2strlqp8s69w"
USR="juno183ct2qqalrkch350zyqwesut7mc976ypj3k6yt"
RM='{"remove_task":{"task_hash":"52bd1f046c2084a04d0f08252d7863b41cb3dfd6e9415d2bf8ce0bcd1a10d55c"}}'
junod tx wasm execute $CONTRACT "$RM" --amount 1000000ujunox --from $USR $TXFLAG -y
