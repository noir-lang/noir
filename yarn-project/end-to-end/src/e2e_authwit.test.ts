import { type AccountWallet, Fr, computeInnerAuthWitHash, computeOuterAuthWitHash } from '@aztec/aztec.js';
import { SchnorrAccountContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { DUPLICATE_NULLIFIER_ERROR } from './fixtures/fixtures.js';
import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 90_000;

describe('e2e_authwit_tests', () => {
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWallet[];

  let chainId: Fr;
  let version: Fr;

  beforeAll(async () => {
    ({ wallets } = await setup(2));
    // docs:start:public_deploy_accounts
    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));
    // docs:end:public_deploy_accounts

    const nodeInfo = await wallets[0].getNodeInfo();
    chainId = new Fr(nodeInfo.chainId);
    version = new Fr(nodeInfo.protocolVersion);
  });

  describe('Private', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        // docs:start:compute_inner_authwit_hash
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead')]);
        // docs:end:compute_inner_authwit_hash
        // docs:start:compute_outer_authwit_hash
        const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);
        // docs:end:compute_outer_authwit_hash
        // docs:start:create_authwit
        const witness = await wallets[0].createAuthWit(outerHash);
        // docs:end:create_authwit
        await wallets[1].addAuthWitness(witness);

        // Check that the authwit is valid in private for wallets[0]
        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
          isValidInPrivate: true,
          isValidInPublic: false,
        });

        // Check that the authwit is NOT valid in private for wallets[1]
        expect(await wallets[0].lookupValidity(wallets[1].getAddress(), outerHash)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });

        const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
        await c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send().wait();

        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });
      });

      describe('failure case', () => {
        it('cancel before usage', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const witness = await wallets[0].createAuthWit(outerHash);
          await wallets[1].addAuthWitness(witness);
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: true,
            isValidInPublic: false,
          });
          await wallets[0].cancelAuthWit(outerHash).send().wait();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          // The transaction should be dropped because of a cancelled authwit (duplicate nullifier)
          await expect(txCancelledAuthwit.wait()).rejects.toThrow(DUPLICATE_NULLIFIER_ERROR);
        });

        it('invalid chain id', async () => {
          const invalidChainId = Fr.random();

          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), invalidChainId, version, innerHash);
          const outerCorrectHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const witness = await wallets[0].createAuthWit(outerHash);
          await wallets[1].addAuthWitness(witness);
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: true,
            isValidInPublic: false,
          });
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: true,
            isValidInPublic: false,
          });
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          // The transaction should be dropped because of the invalid chain id
          await expect(txCancelledAuthwit.wait()).rejects.toThrow(DUPLICATE_NULLIFIER_ERROR);
        });

        it('invalid version', async () => {
          const invalidVersion = Fr.random();

          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, invalidVersion, innerHash);
          const outerCorrectHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const witness = await wallets[0].createAuthWit(outerHash);
          await wallets[1].addAuthWitness(witness);
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: true,
            isValidInPublic: false,
          });
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_private_authwit(innerHash).send();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: true,
            isValidInPublic: false,
          });
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerCorrectHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          // The transaction should be dropped because of the invalid version
          await expect(txCancelledAuthwit.wait()).rejects.toThrow(DUPLICATE_NULLIFIER_ERROR);
        });
      });
    });
  });

  describe('Public', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x01')]);
        const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);
        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });

        // docs:start:set_public_authwit
        await wallets[0].setPublicAuthWit(outerHash, true).send().wait();
        // docs:end:set_public_authwit
        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: true,
        });

        const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
        await c.withWallet(wallets[1]).methods.spend_public_authwit(innerHash).send().wait();

        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });
      });

      describe('failure case', () => {
        it('cancel before usage', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x02')]);
          const outerHash = computeOuterAuthWitHash(wallets[1].getAddress(), chainId, version, innerHash);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          await wallets[0].setPublicAuthWit(outerHash, true).send().wait();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: true,
          });

          await wallets[0].cancelAuthWit(outerHash).send().wait();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), outerHash)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const c = await SchnorrAccountContract.at(wallets[0].getAddress(), wallets[0]);
          const txCancelledAuthwit = c.withWallet(wallets[1]).methods.spend_public_authwit(innerHash).send();
          // The transaction should be dropped because of a cancelled authwit (duplicate nullifier)
          await expect(txCancelledAuthwit.wait()).rejects.toThrow(DUPLICATE_NULLIFIER_ERROR);
        });
      });
    });
  });
});
