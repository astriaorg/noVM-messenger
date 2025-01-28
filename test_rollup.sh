#!/usr/bin/env bash
pre_req=true
rollup_url="http://localhost:3030"
if ! curl -s --head "$rollup_url" > /dev/null; then
    echo "Rollup is not running"
    pre_req=false
fi
sequencer_url="http://127.0.0.1:26657"
if ! curl -s --head "$sequencer_url" > /dev/null; then
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

export PRIV_KEY="2bd806c97f0e00af1a1fc3328fa763a9269723c8db8fac4f93af71db186d6e90"

# echo "Sending Messages"

# rollup-cli query texts --rollup-url $rollup_url
# rollup-cli submit text --private-key $PRIV_KEY --rollup-url $rollup_url "hi" "alice"
# sleep 5
# rollup-cli query texts --rollup-url $rollup_url

echo "Sending Deposit"

rollup-cli query balance --rollup-url $rollup_url astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm

astria-cli sequencer init-bridge-account --private-key $PRIV_KEY --sequencer-url $sequencer_url \
    --sequencer.chain-id sequencer-test-chain-0 \
    --rollup-name astria-chat

astria-cli sequencer bridge-lock astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm \
    --amount 1 \
    --destination-chain-address astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm \
    --private-key $PRIV_KEY  \
    --sequencer.chain-id sequencer-test-chain-0 \
    --sequencer-url $sequencer_url \
    --fee-asset=nria \
    --asset=nria

sleep 5
rollup-cli query balance --rollup-url $rollup_url astria1rsxyjrcm255ds9euthjx6yc3vrjt9sxrm9cfgm
