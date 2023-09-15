import { L2BlockL2Logs, createAztecRpcClient, getSandboxAccountsWallets } from '@aztec/aztec.js';
import { fileURLToPath } from '@aztec/foundation/url';

import { getPrivateToken, getPublicToken } from './contracts.mjs';

const { SANDBOX_URL = 'http://localhost:8080' } = process.env;

async function showAccounts(client) {
  // docs:start:showAccounts
  const accounts = await client.getRegisteredAccounts();
  console.log(`User accounts:\n${accounts.map(a => a.address).join('\n')}`);
  // docs:end:showAccounts
}

async function showPrivateBalances(client) {
  // docs:start:showPrivateBalances
  const accounts = await client.getRegisteredAccounts();
  const privateToken = await getPrivateToken(client);

  for (const account of accounts) {
    // highlight-next-line:showPrivateBalances
    const balance = await privateToken.methods.getBalance(account.address).view();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPrivateBalances
}

async function transferPrivateFunds(client) {
  // docs:start:transferPrivateFunds
  const [owner, recipient] = await getSandboxAccountsWallets(client);
  const privateToken = await getPrivateToken(owner);

  const tx = privateToken.methods.transfer(1n, recipient.getAddress()).send();
  console.log(`Sent transfer transaction ${await tx.getTxHash()}`);
  await showPrivateBalances(client);

  console.log(`Awaiting transaction to be mined`);
  const receipt = await tx.wait();
  console.log(`Transaction has been mined on block ${receipt.blockNumber}`);
  await showPrivateBalances(client);
  // docs:end:transferPrivateFunds
}

async function showPublicBalances(client) {
  // docs:start:showPublicBalances
  const accounts = await client.getRegisteredAccounts();
  const publicToken = await getPublicToken(client);

  for (const account of accounts) {
    // highlight-next-line:showPublicBalances
    const balance = await publicToken.methods.publicBalanceOf(account.address).view();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPublicBalances
}

async function mintPublicFunds(client) {
  // docs:start:mintPublicFunds
  const [owner] = await getSandboxAccountsWallets(client);
  const publicToken = await getPublicToken(owner);

  const tx = publicToken.methods.mint(100n, owner.getAddress()).send();
  console.log(`Sent mint transaction ${await tx.getTxHash()}`);
  await showPublicBalances(client);

  console.log(`Awaiting transaction to be mined`);
  const receipt = await tx.wait();
  console.log(`Transaction has been mined on block ${receipt.blockNumber}`);
  await showPublicBalances(client);
  // docs:end:mintPublicFunds

  // docs:start:showLogs
  const blockNumber = await client.getBlockNumber();
  const logs = await client.getUnencryptedLogs(blockNumber, 1);
  const textLogs = L2BlockL2Logs.unrollLogs(logs).map(log => log.toString('ascii'));
  for (const log of textLogs) console.log(`Log emitted: ${log}`);
  // docs:end:showLogs
}

async function main() {
  const client = createAztecRpcClient(SANDBOX_URL);
  const { chainId } = await client.getNodeInfo();
  console.log(`Connected to chain ${chainId}`);

  await showAccounts(client);

  await transferPrivateFunds(client);

  await mintPublicFunds(client);
}

// Execute main only if run directly
if (process.argv[1].replace(/\/index\.m?js$/, '') === fileURLToPath(import.meta.url).replace(/\/index\.m?js$/, '')) {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  main()
    .then(() => process.exit(0))
    .catch(err => {
      console.error(`Error in app: ${err}`);
      process.exit(1);
    });
}

export { main };
