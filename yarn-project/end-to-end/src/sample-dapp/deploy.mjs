import { getInitialTestAccountsWallets } from '@aztec/accounts/testing';
import { Contract, createPXEClient, loadContractArtifact } from '@aztec/aztec.js';
import TokenContractJson from '@aztec/noir-contracts.js/artifacts/token_contract-Token' assert { type: 'json' };

import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';

// docs:start:dapp-deploy
const { PXE_URL = 'http://localhost:8080' } = process.env;

async function main() {
  const pxe = createPXEClient(PXE_URL);
  const [ownerWallet] = await getInitialTestAccountsWallets(pxe);
  const ownerAddress = ownerWallet.getCompleteAddress();

  const TokenContractArtifact = loadContractArtifact(TokenContractJson);
  const token = await Contract.deploy(ownerWallet, TokenContractArtifact, [ownerAddress, 'TokenName', 'TKN', 18])
    .send()
    .deployed();

  console.log(`Token deployed at ${token.address.toString()}`);

  const addresses = { token: token.address.toString() };
  writeFileSync('addresses.json', JSON.stringify(addresses, null, 2));
}
// docs:end:dapp-deploy

// Execute main only if run directly
if (process.argv[1].replace(/\/index\.m?js$/, '') === fileURLToPath(import.meta.url).replace(/\/index\.m?js$/, '')) {
  main().catch(err => {
    console.error(`Error in deployment script: ${err}`);
    process.exit(1);
  });
}

export { main as deploy };
