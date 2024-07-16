import { type AccountWallet, Fr, computeAuthWitMessageHash, computeInnerAuthWitHash } from '@aztec/aztec.js';
import { AuthRegistryContract, AuthWitTestContract } from '@aztec/noir-contracts.js';
import { getCanonicalAuthRegistry } from '@aztec/protocol-contracts/auth-registry';

import { jest } from '@jest/globals';

import { DUPLICATE_NULLIFIER_ERROR } from './fixtures/fixtures.js';
import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 90_000;

describe('e2e_authwit_tests', () => {
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWallet[];

  let chainId: Fr;
  let version: Fr;
  let auth: AuthWitTestContract;

  beforeAll(async () => {
    ({ wallets } = await setup(2));
    // docs:start:public_deploy_accounts
    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));
    // docs:end:public_deploy_accounts

    const nodeInfo = await wallets[0].getNodeInfo();
    chainId = new Fr(nodeInfo.l1ChainId);
    version = new Fr(nodeInfo.protocolVersion);

    auth = await AuthWitTestContract.deploy(wallets[0]).send().deployed();
  });

  describe('Private', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        // What are we doing here:
        // 1. We compute an inner hash which is here just a hash of random data
        // 2. We then compute the message hash, which is binding it to a "consumer" (here the "auth" contract)
        // 3. We then create an authwit for this message hash.
        // 4. We add this authwit to the wallet[1]
        // 5. We check that the authwit is valid in private for wallet[0] (check that it is signed by 0)
        // 6. We check that the authwit is NOT valid in private for wallet[1] (check that it is not signed by 1)

        // docs:start:compute_inner_authwit_hash
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead')]);
        // docs:end:compute_inner_authwit_hash
        // docs:start:compute_arbitrary_authwit_hash

        const intent = { consumer: auth.address, innerHash };
        // docs:end:compute_arbitrary_authwit_hash
        // docs:start:create_authwit
        const witness = await wallets[0].createAuthWit(intent);
        // docs:end:create_authwit
        await wallets[1].addAuthWitness(witness);

        // Check that the authwit is valid in private for wallets[0]
        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
          isValidInPrivate: true,
          isValidInPublic: false,
        });

        // Check that the authwit is NOT valid in private for wallets[1]
        expect(await wallets[0].lookupValidity(wallets[1].getAddress(), intent)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });

        // Consume the inner hash using the wallets[0] as the "on behalf of".
        await auth.withWallet(wallets[1]).methods.consume(wallets[0].getAddress(), innerHash).send().wait();

        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });

        // Try to consume the same authwit again, it should fail
        await expect(
          auth.withWallet(wallets[1]).methods.consume(wallets[0].getAddress(), innerHash).send().wait(),
        ).rejects.toThrow(DUPLICATE_NULLIFIER_ERROR);
      });
      describe('failure case', () => {
        it('invalid chain id', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const intent = { consumer: auth.address, innerHash };

          const messageHash = computeAuthWitMessageHash(intent, { chainId: Fr.random(), version });
          const expectedMessageHash = computeAuthWitMessageHash(intent, { chainId, version });

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const witness = await wallets[0].createAuthWit(messageHash);
          await wallets[1].addAuthWitness(witness);

          // We should NOT see it as valid, even though we have the authwit, since the chain id is wrong
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          // The transaction should be dropped because of the invalid chain id
          await expect(
            auth.withWallet(wallets[1]).methods.consume(wallets[0].getAddress(), innerHash).simulate(),
          ).rejects.toThrow(`Unknown auth witness for message hash ${expectedMessageHash.toString()}`);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });
        });

        it('invalid version', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0xbeef')]);
          const intent = { consumer: auth.address, innerHash };

          const messageHash = computeAuthWitMessageHash(intent, { chainId, version: Fr.random() });

          const expectedMessageHash = computeAuthWitMessageHash(intent, { chainId, version });

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const witness = await wallets[0].createAuthWit(messageHash);
          await wallets[1].addAuthWitness(witness);

          // We should NOT see it as valid, even though we have the authwit, since the version is wrong
          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          // The transaction should be dropped because of the invalid version
          await expect(
            auth.withWallet(wallets[1]).methods.consume(wallets[0].getAddress(), innerHash).simulate(),
          ).rejects.toThrow(`Unknown auth witness for message hash ${expectedMessageHash.toString()}`);

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });
        });
      });
    });
  });

  describe('Public', () => {
    describe('arbitrary data', () => {
      it('happy path', async () => {
        const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x01')]);

        const intent = { consumer: wallets[1].getAddress(), innerHash };

        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });

        // docs:start:set_public_authwit
        await wallets[0].setPublicAuthWit(intent, true).send().wait();
        // docs:end:set_public_authwit
        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: true,
        });

        const registry = await AuthRegistryContract.at(getCanonicalAuthRegistry().instance.address, wallets[1]);
        await registry.methods.consume(wallets[0].getAddress(), innerHash).send().wait();

        expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
          isValidInPrivate: false,
          isValidInPublic: false,
        });
      });

      describe('failure case', () => {
        it('cancel before usage', async () => {
          const innerHash = computeInnerAuthWitHash([Fr.fromString('0xdead'), Fr.fromString('0x02')]);
          const intent = { consumer: auth.address, innerHash };

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          await wallets[0].setPublicAuthWit(intent, true).send().wait();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: true,
          });

          await wallets[0].setPublicAuthWit(intent, false).send().wait();

          expect(await wallets[0].lookupValidity(wallets[0].getAddress(), intent)).toEqual({
            isValidInPrivate: false,
            isValidInPublic: false,
          });

          const registry = await AuthRegistryContract.at(getCanonicalAuthRegistry().instance.address, wallets[1]);
          await expect(registry.methods.consume(wallets[0].getAddress(), innerHash).simulate()).rejects.toThrow(
            /unauthorized/,
          );
        });
      });
    });
  });
});
