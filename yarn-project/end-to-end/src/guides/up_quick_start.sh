# Run locally from end-to-end folder while running anvil and sandbox with:
# PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh

set -eux

# docs:start:declare-accounts
ALICE="0x25048e8c1b7dea68053d597ac2d920637c99523651edfb123d0632da785970d0"
BOB="0x115f123bbc6cc6af9890055821cfba23a7c4e8832377a32ccb719a1ba3a86483"
ALICE_PRIVATE_KEY="0x2153536ff6628eee01cf4024889ff977a18d9fa61d0e414422f7681cf085c281"
# docs:end:declare-accounts

# docs:start:deploy
aztec-cli deploy \
  TokenContractAbi \
  --salt 0 \
  --args $ALICE

aztec-cli check-deploy --contract-address 0x2219e810bff6e04abdefce9f91c2d1dd1e4d52fafa602def3c90b77f4331feca

CONTRACT="0x2219e810bff6e04abdefce9f91c2d1dd1e4d52fafa602def3c90b77f4331feca"
# docs:end:deploy

# docs:start:mint-private
SECRET="0x29bf6afaf29f61cbcf2a4fa7da97be481fb418dc08bdab5338839974beb7b49f"
SECRET_HASH="0x0a42b1fe22b652cc8610e33bb1128040ce2d2862e7041ff235aa871739822b74"

MINT_PRIVATE_OUTPUT=$(aztec-cli send mint_private \
  --args 1000 $SECRET_HASH \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY)

MINT_PRIVATE_TX_HASH=$(echo "$MINT_PRIVATE_OUTPUT" | grep "Transaction hash:" | awk '{print $NF}')

aztec-cli add-note \
  $ALICE $CONTRACT 5 $MINT_PRIVATE_TX_HASH \
  --preimage 1000 $SECRET_HASH

aztec-cli send redeem_shield \
  --args $ALICE 1000 $SECRET \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY
# docs:end:mint-private

# docs:start:get-balance
aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT
# docs:end:get-balance

# docs:start:transfer
aztec-cli send transfer \
  --args $ALICE $BOB 500 0 \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT \
  --private-key $ALICE_PRIVATE_KEY

aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT

aztec-cli call balance_of_private \
  --args $BOB \
  --contract-abi TokenContractAbi \
  --contract-address $CONTRACT
# docs:end:transfer

aztec-cli get-logs

# Test end result
BOB_BALANCE=$(aztec-cli call balance_of_private --args $BOB --contract-abi TokenContractAbi --contract-address $CONTRACT)
if ! echo $BOB_BALANCE | grep -q 500; then
  echo "Incorrect Bob balance after transaction (expected 500 but got $BOB_BALANCE)"
  exit 1
fi
