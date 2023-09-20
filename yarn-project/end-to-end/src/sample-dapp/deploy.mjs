import { Contract, ContractDeployer, createAztecRpcClient, getSandboxAccountsWallets } from '@aztec/aztec.js';
import { TokenContractAbi } from '@aztec/noir-contracts/artifacts';

import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';

// docs:start:dapp-deploy
const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

async function main() {
  const client = createAztecRpcClient(SANDBOX_URL);
  const [owner] = await getSandboxAccountsWallets(client);

  const token = await Contract.deploy(client, TokenContractAbi, []).send().deployed();
  await token.withWallet(owner).methods._initialize(owner.getAddress()).send().wait();

  console.log(`Token deployed at ${token.address.toString()}`);

  const addresses = {
    token: token.address.toString(),
  };
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
