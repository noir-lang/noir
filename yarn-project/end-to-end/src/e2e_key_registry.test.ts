import { type AccountWallet, AztecAddress, Fr, type PXE } from '@aztec/aztec.js';
import { GeneratorIndex } from '@aztec/circuits.js';
import { poseidon2Hash } from '@aztec/foundation/crypto';
import { KeyRegistryContract, TestContract } from '@aztec/noir-contracts.js';

import { jest } from '@jest/globals';

import { publicDeployAccounts, setup } from './fixtures/utils.js';

const TIMEOUT = 100_000;

describe('SharedMutablePrivateGetter', () => {
  let keyRegistry: KeyRegistryContract;
  let testContract: TestContract;
  let pxe: PXE;
  jest.setTimeout(TIMEOUT);

  let wallets: AccountWallet[];

  let teardown: () => Promise<void>;

  beforeAll(async () => {
    ({ teardown, pxe, wallets } = await setup(2));
    testContract = await TestContract.deploy(wallets[0]).send().deployed();
    keyRegistry = await KeyRegistryContract.deploy(wallets[0]).send().deployed();

    await publicDeployAccounts(wallets[0], wallets.slice(0, 2));
  }, 120_000);

  const delay = async (blocks: number) => {
    for (let i = 0; i < blocks; i++) {
      await testContract.methods.delay().send().wait();
    }
  };

  afterAll(() => teardown());

  describe('failure cases', () => {
    let accountAddedToRegistry: AztecAddress;

    describe('should fail registering with bad input', () => {
      const partialAddress = new Fr(69);

      const masterNullifierPublicKey = new Fr(12);
      const masterIncomingViewingPublicKey = new Fr(34);
      const masterOutgoingViewingPublicKey = new Fr(56);
      const masterTaggingPublicKey = new Fr(78);

      // TODO(#5726): use computePublicKeysHash function
      const publicKeysHash = poseidon2Hash([
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
        GeneratorIndex.PUBLIC_KEYS_HASH,
      ]);

      // We hash the partial address and the public keys hash to get the account address
      // TODO(#5726): Move the following line to AztecAddress class?
      accountAddedToRegistry = poseidon2Hash([partialAddress, publicKeysHash, GeneratorIndex.CONTRACT_ADDRESS_V1]);

      it('should fail registering with mismatched address', async () => {
        const mismatchedAddress = Fr.random();

        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.register(
              AztecAddress.fromField(mismatchedAddress),
              partialAddress,
              masterNullifierPublicKey,
              masterIncomingViewingPublicKey,
              masterOutgoingViewingPublicKey,
              masterTaggingPublicKey,
            )
            .send()
            .wait(),
        ).rejects.toThrow('Computed address does not match supplied address');
      });

      it('should fail registering with mismatched nullifier public key', async () => {
        const mismatchedMasterNullifierPublicKey = Fr.random();

        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.register(
              AztecAddress.fromField(accountAddedToRegistry),
              partialAddress,
              mismatchedMasterNullifierPublicKey,
              masterIncomingViewingPublicKey,
              masterOutgoingViewingPublicKey,
              masterTaggingPublicKey,
            )
            .send()
            .wait(),
        ).rejects.toThrow('Computed address does not match supplied address');
      });
    });

    describe('should fail when rotating keys with bad input', () => {
      it('should fail when trying to rotate setting a 0 key', async () => {
        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.rotate_nullifier_public_key(wallets[0].getAddress(), new Fr(0))
            .send()
            .wait(),
        ).rejects.toThrow('New nullifier public key must be non-zero');
      });

      it('should fail when trying to rotate for another address without authwit', async () => {
        await expect(
          keyRegistry
            .withWallet(wallets[0])
            .methods.rotate_nullifier_public_key(wallets[1].getAddress(), new Fr(2))
            .send()
            .wait(),
        ).rejects.toThrow('Assertion failed: Message not authorized by account');
      });
    });
  });

  describe('key registration flow', () => {
    let accountAddedToRegistry: AztecAddress;

    it('should generate and register with original keys', async () => {
      const partialAddress = new Fr(69);

      const masterNullifierPublicKey = new Fr(12);
      const masterIncomingViewingPublicKey = new Fr(34);
      const masterOutgoingViewingPublicKey = new Fr(56);
      const masterTaggingPublicKey = new Fr(78);

      const publicKeysHash = poseidon2Hash([
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
        GeneratorIndex.PUBLIC_KEYS_HASH,
      ]);

      // We hash the partial address and the public keys hash to get the account address
      // TODO(#5726): Move the following line to AztecAddress class?
      accountAddedToRegistry = poseidon2Hash([partialAddress, publicKeysHash, GeneratorIndex.CONTRACT_ADDRESS_V1]);

      await keyRegistry
        .withWallet(wallets[0])
        .methods.register(
          AztecAddress.fromField(accountAddedToRegistry),
          partialAddress,
          masterNullifierPublicKey,
          masterIncomingViewingPublicKey,
          masterOutgoingViewingPublicKey,
          masterTaggingPublicKey,
        )
        .send()
        .wait();
    });

    it('checks our registry contract from test contract and fails because the address has not been registered yet', async () => {
      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, accountAddedToRegistry)
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });
      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(Fr.ZERO);
    });

    it('checks our registry contract from test contract and finds the address and associated nullifier public key after a delay', async () => {
      await delay(5);

      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, accountAddedToRegistry)
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });

      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(new Fr(12));
    });
  });

  describe('key rotation flow', () => {
    it('we rotate the nullifier key', async () => {
      // This changes
      const newMasterNullifierPublicKey = new Fr(910);

      await keyRegistry
        .withWallet(wallets[0])
        .methods.rotate_nullifier_public_key(wallets[0].getAddress(), newMasterNullifierPublicKey)
        .send()
        .wait();
    });

    it("checks our registry contract from test contract and finds our old public key because the key rotation hasn't been applied yet", async () => {
      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, wallets[0].getAddress())
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });
      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(new Fr(0));
    });

    it('checks our registry contract from test contract and finds the new nullifier public key that has been rotated', async () => {
      await delay(5);

      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, wallets[0].getAddress())
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });

      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(new Fr(910));
    });
  });

  describe('key rotation flow with authwit', () => {
    it('wallet 0 lets wallet 1 call rotate_nullifier_public_key on his behalf with a pre-defined new public key', async () => {
      // This changes
      const newMasterNullifierPublicKey = new Fr(420);

      const action = keyRegistry
        .withWallet(wallets[1])
        .methods.rotate_nullifier_public_key(wallets[0].getAddress(), newMasterNullifierPublicKey);

      await wallets[0]
        .setPublicAuthWit({ caller: wallets[1].getCompleteAddress().address, action }, true)
        .send()
        .wait();

      await action.send().wait();
    });

    it("checks our registry contract from test contract and finds our old public key because the key rotation hasn't been applied yet", async () => {
      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, wallets[0].getAddress())
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });
      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(new Fr(910));
    });

    it('checks our registry contract from test contract and finds the new nullifier public key that has been rotated', async () => {
      await delay(5);

      const { txHash } = await testContract.methods
        .test_shared_mutable_private_getter_for_registry_contract(keyRegistry.address, 1, wallets[0].getAddress())
        .send()
        .wait();

      const rawLogs = await pxe.getUnencryptedLogs({ txHash });

      expect(Fr.fromBuffer(rawLogs.logs[0].log.data)).toEqual(new Fr(420));
    });
  });
});
