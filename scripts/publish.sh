#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

# these are imported by other packages
BASE_PACKAGES="mod-sdk croncat-sdk-core croncat-sdk-factory"
ALL_PACKAGES="croncat-sdk-tasks croncat-sdk-manager croncat-sdk-agents"

# these are imported by other contracts
BASE_CONTRACTS="mod-balances mod-generic"
ALL_CONTRACTS="mod-nft croncat-agents croncat-factory croncat-manager croncat-tasks"

# these are imported by other apps
ALL_INTEGRATION_SDKS="croncat-errors-macro croncat-integration-testing croncat-integration-utils"

SLEEP_TIME=30

for pack in $BASE_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish --allow-dirty
  )
done

# wait for these to be processed on crates.io
echo "Waiting for publishing base packages"
sleep $SLEEP_TIME

for cont in $BASE_CONTRACTS; do
  (
    cd "contracts/$cont"
    echo "Publishing $cont"
    cargo publish --allow-dirty
  )
done

# wait for these to be processed on crates.io
echo "Waiting for publishing base contracts"
sleep $SLEEP_TIME

for pack in $ALL_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish --allow-dirty
  )
done

# wait for these to be processed on crates.io
echo "Waiting for publishing remaining packages"
sleep $SLEEP_TIME

for cont in $ALL_CONTRACTS; do
  (
    cd "contracts/$cont"
    echo "Publishing $cont"
    cargo publish --allow-dirty
  )
done

# wait for these to be processed on crates.io
echo "Waiting for publishing remaining contracts"
sleep $SLEEP_TIME

for sdk in $ALL_INTEGRATION_SDKS; do
  (
    cd "integration-sdk/$sdk"
    echo "Publishing $sdk"
    cargo publish --allow-dirty
  )
done

echo "😻 Everything is published! 🚀"
