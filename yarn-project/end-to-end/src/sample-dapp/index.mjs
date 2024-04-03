import { getInitialTestAccountsWallets } from '@aztec/accounts/testing';
import { ExtendedNote, Fr, Note, computeMessageSecretHash, createPXEClient } from '@aztec/aztec.js';
import { fileURLToPath } from '@aztec/foundation/url';

import { getToken } from './contracts.mjs';

const { PXE_URL = 'http://localhost:8080' } = process.env;

async function showAccounts(pxe) {
  // docs:start:showAccounts
  const accounts = await pxe.getRegisteredAccounts();
  console.log(`User accounts:\n${accounts.map(a => a.address).join('\n')}`);
  // docs:end:showAccounts
}

async function showPrivateBalances(pxe) {
  // docs:start:showPrivateBalances
  const accounts = await pxe.getRegisteredAccounts();
  const token = await getToken(pxe);

  for (const account of accounts) {
    // highlight-next-line:showPrivateBalances
    const balance = await token.methods.balance_of_private(account.address).simulate();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPrivateBalances
}

async function mintPrivateFunds(pxe) {
  const [owner] = await getInitialTestAccountsWallets(pxe);
  const token = await getToken(owner);

  await showPrivateBalances(pxe);

  const mintAmount = 20n;
  const secret = Fr.random();
  const secretHash = await computeMessageSecretHash(secret);
  const receipt = await token.methods.mint_private(mintAmount, secretHash).send().wait();

  const storageSlot = new Fr(5);
  const noteTypeId = new Fr(84114971101151129711410111011678111116101n); // TransparentNote

  const note = new Note([new Fr(mintAmount), secretHash]);
  const extendedNote = new ExtendedNote(
    note,
    owner.getAddress(),
    token.address,
    storageSlot,
    noteTypeId,
    receipt.txHash,
  );
  await pxe.addNote(extendedNote);

  await token.methods.redeem_shield(owner.getAddress(), mintAmount, secret).send().wait();

  await showPrivateBalances(pxe);
}

async function transferPrivateFunds(pxe) {
  // docs:start:transferPrivateFunds
  const [owner, recipient] = await getInitialTestAccountsWallets(pxe);
  const token = await getToken(owner);

  const tx = token.methods.transfer(owner.getAddress(), recipient.getAddress(), 1n, 0).send();
  console.log(`Sent transfer transaction ${await tx.getTxHash()}`);
  await showPrivateBalances(pxe);

  console.log(`Awaiting transaction to be mined`);
  const receipt = await tx.wait();
  console.log(`Transaction has been mined on block ${receipt.blockNumber}`);
  await showPrivateBalances(pxe);
  // docs:end:transferPrivateFunds
}

async function showPublicBalances(pxe) {
  // docs:start:showPublicBalances
  const accounts = await pxe.getRegisteredAccounts();
  const token = await getToken(pxe);

  for (const account of accounts) {
    // highlight-next-line:showPublicBalances
    const balance = await token.methods.balance_of_public(account.address).simulate();
    console.log(`Balance of ${account.address}: ${balance}`);
  }
  // docs:end:showPublicBalances
}

async function mintPublicFunds(pxe) {
  // docs:start:mintPublicFunds
  const [owner] = await getInitialTestAccountsWallets(pxe);
  const token = await getToken(owner);

  const tx = token.methods.mint_public(owner.getAddress(), 100n).send();
  console.log(`Sent mint transaction ${await tx.getTxHash()}`);
  await showPublicBalances(pxe);

  console.log(`Awaiting transaction to be mined`);
  const receipt = await tx.wait();
  console.log(`Transaction has been mined on block ${receipt.blockNumber}`);
  await showPublicBalances(pxe);
  // docs:end:mintPublicFunds

  // docs:start:showLogs
  const blockNumber = await pxe.getBlockNumber();
  const logs = (await pxe.getUnencryptedLogs(blockNumber, 1)).logs;
  const textLogs = logs.map(extendedLog => extendedLog.toHumanReadable().slice(0, 200));
  for (const log of textLogs) console.log(`Log emitted: ${log}`);
  // docs:end:showLogs
}

async function main() {
  const pxe = createPXEClient(PXE_URL);
  const { chainId } = await pxe.getNodeInfo();
  console.log(`Connected to chain ${chainId}`);

  await showAccounts(pxe);

  await mintPrivateFunds(pxe);

  await transferPrivateFunds(pxe);

  await mintPublicFunds(pxe);
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
