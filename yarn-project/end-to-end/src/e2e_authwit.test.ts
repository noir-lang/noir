import { AccountWallet, CompleteAddress, Fr, computeInnerAuthWitHash, computeOuterAuthWitHash } from '@aztec/aztec.js';
import { SchnorrAccountContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 90_000;

describe('e2e_authwit_tests', () => {
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWallet[];
  let accounts: CompleteAddress[];

  beforeAll(async () => {
    ({ wallets, accounts } = await setup(2));
    await publicDeployAccounts(wallets[0], accounts.slice(0, 2));
  }, 100_000);

  describe('Private', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead')]);
        const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), innerHash);

        const witness = await wallets[0].createAuthWitness(outerHash);
        await wallets[1].addAuthWitness(witness);

        const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
        await c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send().wait();
      });

      describe('failure case', () => {
        it('cancel before usage', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), innerHash);

          const witness = await wallets[0].createAuthWitness(outerHash);
          await wallets[1].addAuthWitness(witness);
          await wallets[0].cancelAuthWit(outerHash).send().wait();

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send();
          // The transaction should be dropped because of a cancelled authwit (duplicate nullifier)
          await expect(txCancelledAuthwit.wait()).rejects.toThrowError('Transaction ');
        });
      });
    });
  });

  describe('Public', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x01')]);
        const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), innerHash);

        await wallets[0].setPublicAuth(outerHash, true).send().wait();

        const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
        await c.withWallet(wallets[1]).methods.spend_public_authwit(innerHash).send().wait();
      });

      describe('failure case', () => {
        it('cancel before usage', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x02')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), innerHash);

          await wallets[0].setPublicAuth(outerHash, true).send().wait();

          await wallets[0].cancelAuthWit(outerHash).send().wait();

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_public_authwit(innerHash).send();
          // The transaction should be dropped because of a cancelled authwit (duplicate nullifier)
          await expect(txCancelledAuthwit.wait()).rejects.toThrowError('Transaction ');
        });
      });
    });
  });
});
