import { getSchnorrAccount } from '@aztec/accounts/schnorr';
import {
  type AccountWallet,
  type AccountWalletWithSecretKey,
  type AztecAddress,
  type CompleteAddress,
  ExtendedNote,
  Fr,
  Note,
  type PXE,
  type TxHash,
  computeSecretHash,
  createCompatibleClient,
} from '@aztec/aztec.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { TokenContract } from '@aztec/noir-contracts.js';

import { addAccounts, publicDeployAccounts } from '../fixtures/snapshot_manager.js';

const { PXE_URL } = process.env;
if (!PXE_URL) {
  throw new Error('PXE_URL env variable must be set');
}

const toString = ({ value }: { value: bigint }) => {
  const vals: number[] = Array.from(new Fr(value).toBuffer());

  let str = '';
  for (let i = 0; i < vals.length; i++) {
    if (vals[i] != 0) {
      str += String.fromCharCode(Number(vals[i]));
    }
  }
  return str;
};

const addPendingShieldNoteToPXE = async (args: {
  amount: bigint;
  secretHash: Fr;
  txHash: TxHash;
  accountAddress: AztecAddress;
  assetAddress: AztecAddress;
  wallet: AccountWallet;
}) => {
  const { accountAddress, assetAddress, amount, secretHash, txHash, wallet } = args;
  const note = new Note([new Fr(amount), secretHash]);
  const extendedNote = new ExtendedNote(
    note,
    accountAddress,
    assetAddress,
    TokenContract.storage.pending_shields.slot,
    TokenContract.notes.TransparentNote.id,
    txHash,
  );
  await wallet.addNote(extendedNote);
};

describe('token transfer test', () => {
  const logger = createDebugLogger(`aztec:spartan-test:transfer`);
  const TOKEN_NAME = 'USDC';
  const TOKEN_SYMBOL = 'USD';
  const TOKEN_DECIMALS = 18n;
  const MINT_AMOUNT = 1000000n;
  let pxe: PXE;
  let wallets: AccountWalletWithSecretKey[];
  let completeAddresses: CompleteAddress[];
  let tokenAddress: AztecAddress;
  let tokenAtWallet0: TokenContract;
  beforeAll(async () => {
    pxe = await createCompatibleClient(PXE_URL, logger);
    const { accountKeys } = await addAccounts(3, logger)({ pxe });
    const accountManagers = accountKeys.map(ak => getSchnorrAccount(pxe, ak[0], ak[1], 1));
    wallets = await Promise.all(accountManagers.map(a => a.getWallet()));
    completeAddresses = await pxe.getRegisteredAccounts();
    wallets.forEach((w, i) => logger.verbose(`Wallet ${i} address: ${w.getAddress()}`));
    await publicDeployAccounts(wallets[0], completeAddresses.slice(0, 2));

    logger.verbose(`Deploying TokenContract...`);
    const tokenContract = await TokenContract.deploy(
      wallets[0],
      completeAddresses[0],
      TOKEN_NAME,
      TOKEN_SYMBOL,
      TOKEN_DECIMALS,
    )
      .send()
      .deployed();

    tokenAddress = tokenContract.address;
    tokenAtWallet0 = await TokenContract.at(tokenAddress, wallets[0]);

    logger.verbose(`Minting ${MINT_AMOUNT} publicly...`);
    await tokenAtWallet0.methods.mint_public(completeAddresses[0].address, MINT_AMOUNT).send().wait();

    logger.verbose(`Minting ${MINT_AMOUNT} privately...`);
    const secret = Fr.random();
    const secretHash = computeSecretHash(secret);
    const receipt = await tokenAtWallet0.methods.mint_private(MINT_AMOUNT, secretHash).send().wait();

    await addPendingShieldNoteToPXE({
      amount: MINT_AMOUNT,
      secretHash,
      txHash: receipt.txHash,
      accountAddress: completeAddresses[0].address,
      assetAddress: tokenAddress,
      wallet: wallets[0],
    });
    const txClaim = tokenAtWallet0.methods.redeem_shield(completeAddresses[0].address, MINT_AMOUNT, secret).send();
    await txClaim.wait({ debug: true });
    logger.verbose(`Minting complete.`);
  });

  it('can get info', async () => {
    const name = toString(await tokenAtWallet0.methods.private_get_name().simulate());
    expect(name).toBe(TOKEN_NAME);
  });

  it('can transfer 1 publicly', async () => {
    const transferAmount = 1n;
    const balance0 = await tokenAtWallet0.methods.balance_of_public(completeAddresses[0].address).simulate();
    expect(balance0).toBeGreaterThanOrEqual(transferAmount);
    await tokenAtWallet0.methods
      .transfer_public(completeAddresses[0].address, completeAddresses[1].address, transferAmount, 0)
      .send()
      .wait();
    const balance0After = await tokenAtWallet0.methods.balance_of_public(completeAddresses[0].address).simulate();
    const balance1After = await tokenAtWallet0.methods.balance_of_public(completeAddresses[1].address).simulate();
    expect(balance0After).toBe(balance0 - transferAmount);
    expect(balance1After).toBe(transferAmount);
  });

  it('can transfer 1 privately', async () => {
    const transferAmount = 1n;
    const balance0 = await tokenAtWallet0.methods.balance_of_private(completeAddresses[0].address).simulate();
    expect(balance0).toBeGreaterThanOrEqual(transferAmount);
    await tokenAtWallet0.methods.transfer(completeAddresses[1].address, transferAmount).send().wait();
    const balance0After = await tokenAtWallet0.methods.balance_of_private(completeAddresses[0].address).simulate();
    const balance1After = await tokenAtWallet0.methods.balance_of_private(completeAddresses[1].address).simulate();
    expect(balance0After).toBe(balance0 - transferAmount);
    expect(balance1After).toBe(transferAmount);
  });
});
