#!/bin/bash
set -e
. $SH_DIR/base/init-vars.sh

OWNER_BALANCE=120000
AGENT_BALANCE=10000
USER_BALANCE=15000
FAUCET=cw-croncat-faucet
Signer=$($BINARY keys show signer --address)
echo "Signer: $Signer"

if [[ -z "$Signer" ]]; then
  echo "${Red}Signer is not set. Signer must be set before address initialization ${NoColor}"
  exit 1
fi
echo "Initializing addresses..."
junod keys delete alice -y
ALICE_SEED="legend thunder embrace elegant tonight kid misery tragic merry design produce distance island city cancel shrimp dry eager shop scrub wait cigar tenant carry"
echo $ALICE_SEED | $BINARY keys add alice --recover

junod keys delete bob -y
BOB_SEED="market rent damage chief intact require company female van scout accident amazing thought patch hammer any arch stereo aerobic plastic ranch fluid maple place"
echo $BOB_SEED | $BINARY keys add bob --recover

junod keys delete owner -y
OWNER_SEED="scan quarter purchase hub enlist decade pumpkin young wisdom maple comic tooth surprise caution toe music universe skirt lady income decline sun steel pyramid"
echo $OWNER_SEED | $BINARY keys add owner --recover

junod keys delete agent -y
AGENT_SEED="olive soup parade family educate congress hurt dwarf mom this position hungry unaware aunt swamp sunny analyst wrestle fashion main knife start coffee air"
echo $AGENT_SEED | $BINARY keys add agent --recover

junod keys delete user -y
USER_SEED="fatigue runway knock radio sauce express poem novel will ski various merge dolphin actor immune sea muffin decade pass exclude staff require hazard toe"
echo $USER_SEED | $BINARY keys add user --recover

junod keys delete $FAUCET -y
FAUCET_SEED_PHRASE="very priority voice drink cloud advance wait pave dose useful erode proud just absorb east eyebrow unaware prize old brand above arrow east aim"
$BINARY keys show $FAUCET 2>/dev/null || echo $FAUCET_SEED_PHRASE | $BINARY keys add $FAUCET --recover
sleep 10

ALICE_ADDR=$($BINARY keys show alice --address)
BOB_ADDR=$($BINARY keys show bob --address)
OWNER_ADDR=$($BINARY keys show owner --address)
AGENT_ADDR=$($BINARY keys show agent --address)
USER_ADDR=$($BINARY keys show user --address)
FAUCET_ADDRESS=$($BINARY keys show $FAUCET --address)

echo "${Cyan}Alice :" $ALICE_ADDR "${NoColor}"
echo "${Cyan}Bob :" $BOB_ADDR "${NoColor}"
echo "${Cyan}Owner :" $OWNER_ADDR "${NoColor}"
echo "${Cyan}User :" $USER_ADDR "${NoColor}"
echo "${Cyan}Agent :" $AGENT_ADDR "${NoColor}"
echo "${Cyan}Faucet :" $FAUCET_ADDRESS "${NoColor}"
echo "${Cyan}Signer :" $Signer "${NoColor}"

echo "${Yellow}Sending funds to users...${NoColor}"
$BINARY tx bank send signer $FAUCET_ADDRESS "$OWNER_BALANCE"ujunox $NODE --chain-id $CHAIN_ID --yes --broadcast-mode block --sign-mode direct --fees=6000$STAKE
$BINARY tx bank send $FAUCET $OWNER_ADDR "$OWNER_BALANCE"ujunox $NODE --chain-id $CHAIN_ID --yes --broadcast-mode block --sign-mode direct --fees=6000$STAKE
$BINARY tx bank send $FAUCET $AGENT_ADDR "$AGENT_BALANCE"ujunox $NODE --chain-id $CHAIN_ID --yes --broadcast-mode block --sign-mode direct --fees=6000$STAKE
$BINARY tx bank send $FAUCET $USER_ADDR "$USER_BALANCE"ujunox $NODE --chain-id $CHAIN_ID --yes --broadcast-mode block --sign-mode direct --fees=6000$STAKE
echo "${Cyan}Funds sent...${NoColor}"
