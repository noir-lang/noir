import {
  Fr,
  L2BlockL2Logs,
  computeMessageSecretHash,
  createAztecRpcClient,
  getSandboxAccountsWallets,
} from '@aztec/aztec.js';
import { fileURLToPath } from '@aztec/foundation/url';

import { getToken } from './contracts.mjs';

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
  const token = await getToken(client);

  for (const account of accounts) {
    // highlight-next-line:showPrivateBalances
    const balance = await token.methods.balance_of_private(account.address).view();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPrivateBalances
}

async function mintPrivateFunds(client) {
  const [owner] = await getSandboxAccountsWallets(client);
  const token = await getToken(owner);

  await showPrivateBalances(client);

  const mintAmount = 20n;
  const secret = Fr.random();
  const secretHash = await computeMessageSecretHash(secret);
  await token.methods.mint_private(mintAmount, secretHash).send().wait();
  await token.methods.redeem_shield(owner.getAddress(), mintAmount, secret).send().wait();

  await showPrivateBalances(client);
}

async function transferPrivateFunds(client) {
  // docs:start:transferPrivateFunds
  const [owner, recipient] = await getSandboxAccountsWallets(client);
  const token = await getToken(owner);

  const tx = token.methods.transfer(owner.getAddress(), recipient.getAddress(), 1n, 0).send();
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
  const token = await getToken(client);

  for (const account of accounts) {
    // highlight-next-line:showPublicBalances
    const balance = await token.methods.balance_of_public(account.address).view();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPublicBalances
}

async function mintPublicFunds(client) {
  // docs:start:mintPublicFunds
  const [owner] = await getSandboxAccountsWallets(client);
  const token = await getToken(owner);

  const tx = token.methods.mint_public(owner.getAddress(), 100n).send();
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

  await mintPrivateFunds(client);

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
