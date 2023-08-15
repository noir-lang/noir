# Run locally from end-to-end folder while running anvil and sandbox with:
# PATH=$PATH:../node_modules/.bin ./src/guides/up_quick_start.sh

set -eux

# docs:start:declare-accounts
ALICE="0x2e13f0201905944184fc2c09d29fcf0cac07647be171656a275f63d99b819360"
BOB="0x0d557417a3ce7d7b356a8f15d79a868fd8da2af9c5f4981feb9bcf0b614bd17e"
# docs:end:declare-accounts

# docs:start:deploy
aztec-cli deploy \
  --contract-abi PrivateTokenContractAbi \
  --args 1000000 $ALICE \
  --salt 0
# docs:end:deploy

aztec-cli check-deploy --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f

# docs:start:get-balance
aztec-cli call getBalance \
  --args $ALICE \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f
# docs:end:get-balance

# docs:start:transfer
aztec-cli send transfer \
  --args 500 $ALICE $BOB \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f \
  --private-key 0xb2803ec899f76f6b2ac011480d24028f1a29587f8a3a92f7ee9d48d8c085c284

aztec-cli call getBalance \
  --args $ALICE \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f

aztec-cli call getBalance \
  --args $BOB \
  --contract-abi PrivateTokenContractAbi \
  --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f
# docs:end:transfer

aztec-cli get-logs

# Test end result
BOB_BALANCE=$(aztec-cli call getBalance --args $BOB --contract-abi PrivateTokenContractAbi --contract-address 0x03b030d48607ba8a0562f0f1f82be26c3f091e45e10f74c2d8cebb80d526a69f)
if ! echo $BOB_BALANCE | grep -q 500; then 
  echo "Incorrect Bob balance after transaction (expected 500 but got $BOB_BALANCE)"
  exit 1
fi
