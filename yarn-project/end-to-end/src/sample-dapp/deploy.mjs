import { ContractDeployer, createAztecRpcClient } from '@aztec/aztec.js';
import {
  PrivateTokenContractAbi as PrivateTokenArtifact,
  PublicTokenContractAbi as PublicTokenArtifact,
} from '@aztec/noir-contracts/artifacts';

import { writeFileSync } from 'fs';
import { fileURLToPath } from 'url';

// docs:start:dapp-deploy
const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

async function main() {
  const client = createAztecRpcClient(SANDBOX_URL);
  const [owner] = await client.getAccounts();

  const privateTokenDeployer = new ContractDeployer(PrivateTokenArtifact, client);
  const { contractAddress: privateTokenAddress } = await privateTokenDeployer.deploy(100n, owner.address).send().wait();
  console.log(`Private token deployed at ${privateTokenAddress.toString()}`);

  const publicTokenDeployer = new ContractDeployer(PublicTokenArtifact, client);
  const { contractAddress: publicTokenAddress } = await publicTokenDeployer.deploy().send().wait();
  console.log(`Public token deployed at ${publicTokenAddress.toString()}`);

  const addresses = { privateToken: privateTokenAddress.toString(), publicToken: publicTokenAddress.toString() };
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
