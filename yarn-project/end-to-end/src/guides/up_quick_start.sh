# Run locally from end-to-end folder while running anvil and sandbox with:
# PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh

set -eux

# docs:start:declare-accounts
ACCOUNTS=$(aztec-cli get-accounts --json | jq -r '.[].address')
ALICE=$(echo "$ACCOUNTS" | sed -n 1p)
BOB=$(echo "$ACCOUNTS" | sed -n 2p)
ALICE_PRIVATE_KEY="0x2153536ff6628eee01cf4024889ff977a18d9fa61d0e414422f7681cf085c281"
# docs:end:declare-accounts

# docs:start:deploy
CONTRACT=$(aztec-cli deploy TokenContractArtifact --private-key $ALICE_PRIVATE_KEY --salt 0 --args $ALICE "TokenName" "TKN" 18 --json | jq -r '.address')
echo "Deployed contract at $CONTRACT"
aztec-cli check-deploy --contract-address $CONTRACT
# docs:end:deploy

# docs:start:mint-private
SECRET="0x29bf6afaf29f61cbcf2a4fa7da97be481fb418dc08bdab5338839974beb7b49f"
SECRET_HASH="0x0921759afa747c9073f75df9688a17d271cef0d6ec51eacf70e112402c4db6cd"

MINT_PRIVATE_OUTPUT=$(aztec-cli send mint_private \
  --args 1000 $SECRET_HASH \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY)

MINT_PRIVATE_TX_HASH=$(echo "$MINT_PRIVATE_OUTPUT" | grep "Transaction hash:" | awk '{print $NF}')

aztec-cli add-note \
  $ALICE $CONTRACT 5 84114971101151129711410111011678111116101 $MINT_PRIVATE_TX_HASH \
  --note 1000 $SECRET_HASH

aztec-cli send redeem_shield \
  --args $ALICE 1000 $SECRET \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY
# docs:end:mint-private

# docs:start:get-balance
aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT
# docs:end:get-balance

# docs:start:transfer
aztec-cli send transfer \
  --args $ALICE $BOB 500 0 \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY

aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT

aztec-cli call balance_of_private \
  --args $BOB \
  --contract-artifact TokenContractArtifact \
  --contract-address $CONTRACT
# docs:end:transfer

aztec-cli get-logs

# Test end result
BOB_BALANCE=$(aztec-cli call balance_of_private --args $BOB --contract-artifact TokenContractArtifact --contract-address $CONTRACT)
if ! echo $BOB_BALANCE | grep -q 500; then
  echo "Incorrect Bob balance after transaction (expected 500 but got $BOB_BALANCE)"
  exit 1
fi
