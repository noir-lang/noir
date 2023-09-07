import { AztecRPCServer } from '@aztec/aztec-rpc';
import {
  Account,
  AccountContract,
  AuthWitnessAccountContract,
  AuthWitnessAccountEntrypoint,
  AuthWitnessEntrypointWallet,
  AztecRPC,
  EcdsaAccountContract,
  Fr,
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
    rpc: AztecRPC,
    encryptionPrivateKey: GrumpkinPrivateKey,
    accountContract: AccountContract,
    address?: CompleteAddress,
  ) => Promise<{ account: Account; wallet: Wallet }>,
) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: ChildContract;
    let account: Account;
    let wallet: Wallet;
    let encryptionPrivateKey: GrumpkinPrivateKey;

    beforeEach(async () => {
      context = await setup(0);
      encryptionPrivateKey = GrumpkinScalar.random();

      ({ account, wallet } = await walletSetup(
        context.aztecRpcServer,
        encryptionPrivateKey,
        getAccountContract(encryptionPrivateKey),
      ));
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
      const { wallet: invalidWallet } = await walletSetup(
        context.aztecRpcServer,
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
    rpc: AztecRPC,
    encryptionPrivateKey: GrumpkinPrivateKey,
    accountContract: AccountContract,
    address?: CompleteAddress,
  ) => {
    const account = new Account(rpc, encryptionPrivateKey, accountContract, address);
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

  describe('eip single-key account', () => {
    itShouldBehaveLikeAnAccountContract(
      (encryptionKey: GrumpkinPrivateKey) => new AuthWitnessAccountContract(encryptionKey),
      async (
        rpc: AztecRPC,
        encryptionPrivateKey: GrumpkinPrivateKey,
        accountContract: AccountContract,
        address?: CompleteAddress,
      ) => {
        const account = new Account(rpc, encryptionPrivateKey, accountContract, address);
        if (!address) {
          const tx = await account.deploy();
          await tx.wait();
        }
        const entryPoint = (await account.getEntrypoint()) as unknown as AuthWitnessAccountEntrypoint;
        const wallet = new AuthWitnessEntrypointWallet(rpc, entryPoint);
        return { account, wallet };
      },
    );
  });
});
