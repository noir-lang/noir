import { AztecRPCServer } from '@aztec/aztec-rpc';
import {
  Account,
  AccountContract,
  EcdsaAccountContract,
  Fr,
  SchnorrAccountContract,
  SingleKeyAccountContract,
  Wallet,
} from '@aztec/aztec.js';
import { PrivateKey } from '@aztec/circuits.js';
import { toBigInt } from '@aztec/foundation/serialize';
import { ChildContract } from '@aztec/noir-contracts/types';

import { setup } from './fixtures/utils.js';

function itShouldBehaveLikeAnAccountContract(getAccountContract: (encryptionKey: PrivateKey) => AccountContract) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: ChildContract;
    let account: Account;
    let wallet: Wallet;
    let encryptionPrivateKey: PrivateKey;

    beforeEach(async () => {
      context = await setup();
      encryptionPrivateKey = PrivateKey.random();
      account = new Account(context.aztecRpcServer, encryptionPrivateKey, getAccountContract(encryptionPrivateKey));
      wallet = await account.deploy().then(tx => tx.getWallet());
      child = await ChildContract.deploy(wallet).send().deployed();
    }, 60_000);

    afterEach(async () => {
      await context.aztecNode?.stop();
      if (context.aztecRpcServer instanceof AztecRPCServer) {
        await context.aztecRpcServer.stop();
      }
    });

    it('calls a private function', async () => {
      const { logger } = context;
      logger('Calling private function...');
      const tx = child.methods.value(42).send();
      expect(await tx.isMined({ interval: 0.1 })).toBeTruthy();
    }, 60_000);

    it('calls a public function', async () => {
      const { logger, aztecRpcServer } = context;
      logger('Calling public function...');
      const tx = child.methods.pubIncValue(42).send();
      expect(await tx.isMined({ interval: 0.1 })).toBeTruthy();
      expect(toBigInt((await aztecRpcServer.getPublicStorageAt(child.address, new Fr(1)))!)).toEqual(42n);
    }, 60_000);

    it('fails to call a function using an invalid signature', async () => {
      const accountAddress = await account.getCompleteAddress();
      const invalidWallet = await new Account(
        context.aztecRpcServer,
        encryptionPrivateKey,
        getAccountContract(PrivateKey.random()),
        accountAddress,
      ).getWallet();
      const childWithInvalidWallet = await ChildContract.at(child.address, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).simulate()).rejects.toThrowError(/Assertion failed: '.*'/);
    });
  });
}

describe('e2e_account_contracts', () => {
  describe('schnorr single-key account', () => {
    itShouldBehaveLikeAnAccountContract((encryptionKey: PrivateKey) => new SingleKeyAccountContract(encryptionKey));
  });

  describe('schnorr multi-key account', () => {
    itShouldBehaveLikeAnAccountContract(() => new SchnorrAccountContract(PrivateKey.random()));
  });

  describe('ecdsa stored-key account', () => {
    itShouldBehaveLikeAnAccountContract(() => new EcdsaAccountContract(PrivateKey.random()));
  });
});
