# Run locally from end-to-end folder while running anvil and sandbox with:
# PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh

set -eux

# docs:start:declare-accounts
ALICE="0x296904eaff80711ddd5fd39d75920bfc22725cc0ae3089f16a09a4ceb91be141"
BOB="0x1106245c5c50798338e55094c38025db4053a25e96dd8cbc1e87c6dcbfb9d2ba"
ALICE_PRIVATE_KEY="0x2153536ff6628eee01cf4024889ff977a18d9fa61d0e414422f7681cf085c281"
# docs:end:declare-accounts

# docs:start:deploy
aztec-cli deploy \
  TokenContractAbi \
  --salt 0

aztec-cli check-deploy --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c

aztec-cli send _initialize \
  --args $ALICE \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c \
  --private-key $ALICE_PRIVATE_KEY
# docs:end:deploy

# docs:start:mint-private
SECRET="0x29bf6afaf29f61cbcf2a4fa7da97be481fb418dc08bdab5338839974beb7b49f"
SECRET_HASH="0x0a42b1fe22b652cc8610e33bb1128040ce2d2862e7041ff235aa871739822b74"

aztec-cli send mint_private \
  --args 1000 $SECRET_HASH \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c \
  --private-key $ALICE_PRIVATE_KEY

aztec-cli send redeem_shield \
  --args $ALICE 1000 $SECRET \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c \
  --private-key $ALICE_PRIVATE_KEY
# docs:end:mint-private

# docs:start:get-balance
aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c
# docs:end:get-balance

# docs:start:transfer
aztec-cli send transfer \
  --args $ALICE $BOB 500 0 \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c \
  --private-key $ALICE_PRIVATE_KEY

aztec-cli call balance_of_private \
  --args $ALICE \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c

aztec-cli call balance_of_private \
  --args $BOB \
  --contract-abi TokenContractAbi \
  --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c
# docs:end:transfer

aztec-cli get-logs

# Test end result
BOB_BALANCE=$(aztec-cli call balance_of_private --args $BOB --contract-abi TokenContractAbi --contract-address 0x2d23acefa3ce07b3c308caf78d86c064cdf8957bcea48b38753cf58441796c8c)
if ! echo $BOB_BALANCE | grep -q 500; then 
  echo "Incorrect Bob balance after transaction (expected 500 but got $BOB_BALANCE)"
  exit 1
fi
