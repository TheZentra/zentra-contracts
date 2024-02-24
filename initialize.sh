#!/bin/bash

set -e

if [[ -f "./.stream-payment-dapp/streamdapp_id" ]]; then
  echo "Found existing './.stream-payment-dapp' directory; already initialized."
  exit 0
fi

SOROBAN_RPC_URL="https://soroban-testnet.stellar.org"
SOROBAN_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
FRIENDBOT_URL="https://friendbot.stellar.org/"
NETWORK="Testnet"

echo "Using $NETWORK network"
echo "  RPC URL: $SOROBAN_RPC_URL"
echo "  Friendbot URL: $FRIENDBOT_URL"

echo Add the $NETWORK network to cli client
soroban config network add \
  --rpc-url "$SOROBAN_RPC_URL" \
  --network-passphrase "$SOROBAN_NETWORK_PASSPHRASE" "$NETWORK"

echo Add $NETWORK to .stream-payment-dapp for use with npm scripts
mkdir -p .stream-payment-dapp
echo $NETWORK > ./.stream-payment-dapp/network
echo $SOROBAN_RPC_URL > ./.stream-payment-dapp/rpc-url
echo "$SOROBAN_NETWORK_PASSPHRASE" > ./.stream-payment-dapp/passphrase

if !(soroban config identity ls | grep token-admin 2>&1 >/dev/null); then
  echo Create the token-admin identity
  soroban config identity generate token-admin
fi
ADMIN_ADDRESS="$(soroban config identity address token-admin)"

# This will fail if the account already exists, but it'll still be fine.
echo Fund token-admin account from friendbot
curl --silent -X POST "$FRIENDBOT_URL?addr=$ADMIN_ADDRESS" >/dev/null

ARGS="--network $NETWORK --source token-admin"

echo Build contracts
make build

echo Deploy the mock token contract
MOCK_TOKEN_ID="$(
  soroban contract deploy $ARGS \
    --wasm target/wasm32-unknown-unknown/release/mock_token.wasm
)"
echo "Contract deployed succesfully with ID: $MOCK_TOKEN_ID"
echo -n "$MOCK_TOKEN_ID" > .stream-payment-dapp/mock_token_id

echo Deploy the streamdapp contract
STREAMDAPP_ID="$(
  soroban contract deploy $ARGS \
    --wasm target/wasm32-unknown-unknown/release/soroban_stream_contract.wasm
)"
echo "Contract deployed succesfully with ID: $STREAMDAPP_ID"
echo "$STREAMDAPP_ID" > .stream-payment-dapp/streamdapp_id

echo "Initialize the mock token contract"
soroban contract invoke \
  $ARGS \
  --id "$MOCK_TOKEN_ID" \
  -- \
  initialize \
  --symbol MCKT \
  --decimal 7 \
  --name mock_token \
  --admin "$ADMIN_ADDRESS"

echo "Minting 1000000000 tokens to token-admin"
soroban contract invoke \
  $ARGS \
  --id "$MOCK_TOKEN_ID" \
  -- \
  mint \
  --to "$ADMIN_ADDRESS" \
  --amount 100000000000000000

echo "Initialize the streamdapp contract"
soroban contract invoke \
  $ARGS \
  --id "$STREAMDAPP_ID" \
  -- \
  initialize \
  --token "$MOCK_TOKEN_ID"
echo "Done"
