#!/bin/bash

set -e

ADMIN_ADDRESS="$(soroban config identity address token-admin)"
NETWORK="$(cat ./.stream-payment-dapp/network)"
MOCK_TOKEN_ID="$(cat ./.stream-payment-dapp/mock_token_id)"

echo Add the network to cli client
soroban config network add \
  --rpc-url "https://soroban-testnet.stellar.org" \
  --network-passphrase "Test SDF Network ; September 2015" "futurenet"

ARGS="--network $NETWORK --source token-admin"

echo Build contracts
make build

echo Deploy the streamdapp contract
STREAMDAPP_ID="$(
  soroban contract deploy $ARGS \
    --wasm target/wasm32-unknown-unknown/release/soroban_stream_contract.wasm
)"
echo "Contract deployed succesfully with ID: $STREAMDAPP_ID"
echo "$STREAMDAPP_ID" > .stream-payment-dapp/streamdapp_id

echo "Initialize the streamdapp contract"
soroban contract invoke \
  $ARGS \
  --id "$STREAMDAPP_ID" \
  -- \
  initialize \
  --token "$MOCK_TOKEN_ID"

echo "Done" 