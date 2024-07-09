import { EcdsaAccountContract } from '@aztec/accounts/ecdsa';
import { SchnorrAccountContract } from '@aztec/accounts/schnorr';
import { SingleKeyAccountContract } from '@aztec/accounts/single_key';
import {
  type AccountContract,
  AccountManager,
  AccountWallet,
  type CompleteAddress,
  type DebugLogger,
  Fr,
  GrumpkinScalar,
  type PXE,
  type Wallet,
} from '@aztec/aztec.js';
import { deriveSigningKey } from '@aztec/circuits.js/keys';
import { randomBytes } from '@aztec/foundation/crypto';
import { ChildContract } from '@aztec/noir-contracts.js/Child';

import { setup } from './fixtures/utils.js';

function itShouldBehaveLikeAnAccountContract(
  getAccountContract: (encryptionKey: GrumpkinScalar) => AccountContract,
  walletSetup: (pxe: PXE, secretKey: Fr, accountContract: AccountContract) => Promise<Wallet>,
  walletAt: (pxe: PXE, accountContract: AccountContract, address: CompleteAddress) => Promise<Wallet>,
) {
  describe(`behaves like an account contract`, () => {
    let child: ChildContract;
    let wallet: Wallet;
    let secretKey: Fr;

    let pxe: PXE;
    let logger: DebugLogger;
    let teardown: () => Promise<void>;

    beforeEach(async () => {
      ({ logger, pxe, teardown } = await setup(0));
      secretKey = Fr.random();
      const signingKey = deriveSigningKey(secretKey);

      wallet = await walletSetup(pxe, secretKey, getAccountContract(signingKey));
      child = await ChildContract.deploy(wallet).send().deployed();
    });

    afterEach(() => teardown());

    it('calls a private function', async () => {
      logger.info('Calling private function...');
      await child.methods.value(42).send().wait({ interval: 0.1 });
    });

    it('calls a public function', async () => {
      logger.info('Calling public function...');
      await child.methods.pub_inc_value(42).send().wait({ interval: 0.1 });
      const storedValue = await pxe.getPublicStorageAt(child.address, new Fr(1));
      expect(storedValue).toEqual(new Fr(42n));
    });

    it('fails to call a function using an invalid signature', async () => {
      const accountAddress = wallet.getCompleteAddress();
      const invalidWallet = await walletAt(pxe, getAccountContract(GrumpkinScalar.random()), accountAddress);
      const childWithInvalidWallet = await ChildContract.at(child.address, invalidWallet);
      await expect(childWithInvalidWallet.methods.value(42).prove()).rejects.toThrow(/Cannot satisfy constraint.*/);
    });
  });
}

describe('e2e_account_contracts', () => {
  const walletSetup = async (pxe: PXE, secretKey: Fr, accountContract: AccountContract) => {
    const account = new AccountManager(pxe, secretKey, accountContract);
    return await account.waitSetup();
  };

  const walletAt = async (pxe: PXE, accountContract: AccountContract, address: CompleteAddress) => {
    const nodeInfo = await pxe.getNodeInfo();
    const entrypoint = accountContract.getInterface(address, nodeInfo);
    return new AccountWallet(pxe, entrypoint);
  };

  describe('schnorr single-key account', () => {
    itShouldBehaveLikeAnAccountContract(
      (encryptionKey: GrumpkinScalar) => new SingleKeyAccountContract(encryptionKey),
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
