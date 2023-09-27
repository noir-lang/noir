import {
  AccountContract,
  AccountManager,
  EcdsaAccountContract,
  Fr,
  PXE,
  SchnorrAccountContract,
  SingleKeyAccountContract,
  Wallet,
} from '@aztec/aztec.js';
import { CompleteAddress, GrumpkinPrivateKey, GrumpkinScalar } from '@aztec/circuits.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract } from '@aztec/noir-contracts/types';

import { randomBytes } from 'crypto';

import { setup } from './fixtures/utils.js';

function itShouldBehaveLikeAnAccountContract(
  getAccountContract: (encryptionKey: GrumpkinPrivateKey) => AccountContract,
  walletSetup: (
    pxe: PXE,
    encryptionPrivateKey: GrumpkinPrivateKey,
    accountContract: AccountContract,
    address?: CompleteAddress,
  ) => Promise<{ account: AccountManager; wallet: Wallet }>,
) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: ChildContract;
    let account: AccountManager;
    let wallet: Wallet;
    let encryptionPrivateKey: GrumpkinPrivateKey;

    beforeEach(async () => {
      context = await setup(0);
      encryptionPrivateKey = GrumpkinScalar.random();

      ({ account, wallet } = await walletSetup(
        context.pxe,
        encryptionPrivateKey,
        getAccountContract(encryptionPrivateKey),
      ));
      child = await ChildContract.deploy(wallet).send().deployed();
    }, 60_000);

    afterEach(() => context.teardown());

    it('calls a private function', async () => {
      const { logger } = context;
      logger('Calling private function...');
      await child.methods.value(42).send().wait({ interval: 0.1 });
    }, 60_000);

    it('calls a public function', async () => {
      const { logger, pxe } = context;
      logger('Calling public function...');
      await child.methods.pubIncValue(42).send().wait({ interval: 0.1 });
      expect(toBigInt((await pxe.getPublicStorageAt(child.address, new Fr(1)))!)).toEqual(42n);
    }, 60_000);

    it('fails to call a function using an invalid signature', async () => {
      const accountAddress = await account.getCompleteAddress();
      const { wallet: invalidWallet } = await walletSetup(
        context.pxe,
        encryptionPrivateKey,
        getAccountContract(GrumpkinScalar.random()),
        accountAddress,
      );
      const childWithInvalidWallet = await ChildContract.at(child.address, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).simulate()).rejects.toThrowError(
        /Cannot satisfy constraint.*/,
      );
    });
  });
}

describe('e2e_account_contracts', () => {
  const base = async (
    pxe: PXE,
    encryptionPrivateKey: GrumpkinPrivateKey,
    accountContract: AccountContract,
    address?: CompleteAddress,
  ) => {
    const account = new AccountManager(pxe, encryptionPrivateKey, accountContract, address);
    const wallet = !address ? await account.deploy().then(tx => tx.getWallet()) : await account.getWallet();
    return { account, wallet };
  };

  describe('schnorr single-key account', () => {
    itShouldBehaveLikeAnAccountContract(
      (encryptionKey: GrumpkinPrivateKey) => new SingleKeyAccountContract(encryptionKey),
      base,
    );
  });

  describe('schnorr multi-key account', () => {
    itShouldBehaveLikeAnAccountContract(() => new SchnorrAccountContract(GrumpkinScalar.random()), base);
  });

  describe('ecdsa stored-key account', () => {
    itShouldBehaveLikeAnAccountContract(() => new EcdsaAccountContract(randomBytes(32)), base);
  });
});
