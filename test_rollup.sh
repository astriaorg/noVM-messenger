#!/usr/bin/env bash
pre_req=true
ROLLUP_URL="http://localhost:3030"
if ! curl -s --head "$ROLLUP_URL" > /dev/null; then
    echo "Rollup is not running"
    pre_req=false
fi
SEQUENCER_URL="http://127.0.0.1:26657"
if ! curl -s --head "$SEQUENCER_URL" > /dev/null; then
    echo "Rollup is not running"
    pre_req=false
fi
if ! which rollup-cli > /dev/null 2>&1; then
    echo "rollup-cli is not installed"
    pre_req=false
fi
if ! which astria-cli > /dev/null 2>&1; then
    echo "astria-cli is not installed"
    pre_req=false
fi
if [ "$pre_req" = false ]; then
    exit 1
fi

BRIDGE_ADDRESS=astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm
BRIDGE_PRIV_KEY=2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90
BOB_ADDRESS=astria1yf56efahcq786pe5t7paknat40g6q4tsvqtql2
BOB_PRIV_KEY=b70fd3b99cab2d98dbd73602deb026b9cdc9bb7b85d35f0bbb81b17c78923dd0
ROLLUP_BRIDGE_ADDRESS=astria1f6yydwp23ucl6kfxt2gqt9vufgpsl3zvz5hwxk
CAROL_ADDRESS=astria1f6yydwp23ucl6kfxt2gqt9vufgpsl3zvz5hwxk
CAROL_PRIV_KEY=0e951afdcbefc420fe6f71b82b0c28c11eb6ee5d95be0886ce9dbf6fa512debc

echo " == Sending Test Message == "

rollup-cli query texts --rollup-url $ROLLUP_URL
rollup-cli submit text --private-key $BOB_PRIV_KEY --rollup-url $ROLLUP_URL "hi" "alice"
sleep 5
rollup-cli query texts --rollup-url $ROLLUP_URL

echo
echo " == Sending Rollup Transfer == "

rollup-cli query balance --rollup-url $ROLLUP_URL --asset nria $BOB_ADDRESS
rollup-cli submit transfer --amount 10 \
    --private-key $BRIDGE_PRIV_KEY \
    --rollup-url $ROLLUP_URL \
    --chain-id astria-chat \
    --asset nria \
    --fee-asset nria \
    $BOB_ADDRESS
sleep 5
rollup-cli query balance --rollup-url $ROLLUP_URL --asset nria $BOB_ADDRESS

echo
echo " == Sending Deposit == "

rollup-cli query balance --rollup-url $ROLLUP_URL --asset ntia $BOB_ADDRESS


echo " -- Transfer -- "
astria-cli sequencer transfer --amount 50 \
    --private-key $BRIDGE_PRIV_KEY  \
    --sequencer.chain-id sequencer-test-chain-0 \
    --sequencer-url $SEQUENCER_URL \
    --fee-asset=ntia \
    --asset=ntia \
    $CAROL_ADDRESS

sleep 5

astria-cli sequencer balance get --sequencer-url $SEQUENCER_URL $CAROL_ADDRESS

echo " -- Init Bridge Account -- "
astria-cli sequencer init-bridge-account --private-key $CAROL_PRIV_KEY --sequencer-url $SEQUENCER_URL \
    --sequencer.chain-id sequencer-test-chain-0 \
    --rollup-name astria-chat \
    --asset ntia \
    --fee-asset ntia

sleep 5

echo " -- Lock -- "
astria-cli sequencer bridge-lock $CAROL_ADDRESS \
    --amount 1 \
    --destination-chain-address $BOB_ADDRESS \
    --private-key $BRIDGE_PRIV_KEY  \
    --sequencer.chain-id sequencer-test-chain-0 \
    --sequencer-url $SEQUENCER_URL \
    --fee-asset=ntia \
    --asset=ntia

sleep 5
rollup-cli query balance --rollup-url $ROLLUP_URL --asset ntia $BOB_ADDRESS

echo
