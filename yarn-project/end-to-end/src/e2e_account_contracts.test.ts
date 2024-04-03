import { EcdsaAccountContract } from '@aztec/accounts/ecdsa';
import { SchnorrAccountContract } from '@aztec/accounts/schnorr';
import { SingleKeyAccountContract } from '@aztec/accounts/single_key';
import {
  type AccountContract,
  AccountManager,
  AccountWallet,
  type CompleteAddress,
  Fr,
  type GrumpkinPrivateKey,
  GrumpkinScalar,
  type PXE,
  type Wallet,
} from '@aztec/aztec.js';
import { randomBytes } from '@aztec/foundation/crypto';
import { ChildContract } from '@aztec/noir-contracts.js/Child';

import { setup } from './fixtures/utils.js';

function itShouldBehaveLikeAnAccountContract(
  getAccountContract: (encryptionKey: GrumpkinPrivateKey) => AccountContract,
  walletSetup: (
    pxe: PXE,
    encryptionPrivateKey: GrumpkinPrivateKey,
    accountContract: AccountContract,
  ) => Promise<Wallet>,
  walletAt: (pxe: PXE, accountContract: AccountContract, address: CompleteAddress) => Promise<Wallet>,
) {
  describe(`behaves like an account contract`, () => {
    let context: Awaited<ReturnType<typeof setup>>;
    let child: ChildContract;
    let wallet: Wallet;
    let encryptionPrivateKey: GrumpkinPrivateKey;

    beforeEach(async () => {
      context = await setup(0);
      encryptionPrivateKey = GrumpkinScalar.random();

      wallet = await walletSetup(context.pxe, encryptionPrivateKey, getAccountContract(encryptionPrivateKey));
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
      await child.methods.pub_inc_value(42).send().wait({ interval: 0.1 });
      const storedValue = await pxe.getPublicStorageAt(child.address, new Fr(1));
      expect(storedValue).toEqual(new Fr(42n));
    }, 60_000);

    it('fails to call a function using an invalid signature', async () => {
      const accountAddress = wallet.getCompleteAddress();
      const invalidWallet = await walletAt(context.pxe, getAccountContract(GrumpkinScalar.random()), accountAddress);
      const childWithInvalidWallet = await ChildContract.at(child.address, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).prove()).rejects.toThrow(/Cannot satisfy constraint.*/);
    });
  });
}

describe('e2e_account_contracts', () => {
  const walletSetup = async (pxe: PXE, encryptionPrivateKey: GrumpkinPrivateKey, accountContract: AccountContract) => {
    const account = new AccountManager(pxe, encryptionPrivateKey, accountContract);
    return await account.waitSetup();
  };

  const walletAt = async (pxe: PXE, accountContract: AccountContract, address: CompleteAddress) => {
    const nodeInfo = await pxe.getNodeInfo();
    const entrypoint = accountContract.getInterface(address, nodeInfo);
    return new AccountWallet(pxe, entrypoint);
  };

  describe('schnorr single-key account', () => {
    itShouldBehaveLikeAnAccountContract(
      (encryptionKey: GrumpkinPrivateKey) => new SingleKeyAccountContract(encryptionKey),
      walletSetup,
      walletAt,
    );
  });

  describe('schnorr multi-key account', () => {
    itShouldBehaveLikeAnAccountContract(
      () => new SchnorrAccountContract(GrumpkinScalar.random()),
      walletSetup,
      walletAt,
    );
  });

  describe('ecdsa stored-key account', () => {
    itShouldBehaveLikeAnAccountContract(() => new EcdsaAccountContract(randomBytes(32)), walletSetup, walletAt);
  });
});
